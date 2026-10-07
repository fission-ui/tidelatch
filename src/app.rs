use std::{collections::HashMap, fmt, time::Duration};

use fission::game::{Game, GameHostInput, GameInputRegion, GameRuntime, GameTime, StepDuration};
use fission::i18n::{Locale, TranslationBundle};
use fission::prelude::*;
use fission::scene2d::{Scene2D, Scene2DIR};

use crate::game::{GameMessage, TideLatchGame, VIEWPORT_SIZE};

pub struct TideLatchState {
    runtime: GameRuntime<TideLatchGame>,
    scene: Scene2DIR,
}

impl TideLatchState {
    fn new() -> Self {
        let game = TideLatchGame::default();
        let scene = game.present(GameTime::default());
        Self {
            runtime: GameRuntime::new(game),
            scene,
        }
    }

    fn advance(&mut self) {
        self.scene = self
            .runtime
            .advance(StepDuration::from_hz(60).as_duration())
            .presentation;
    }

    fn dispatch(&mut self, message: GameMessage) {
        self.runtime.send(message);
        self.advance();
    }
}

impl Default for TideLatchState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TideLatchState {
    fn clone(&self) -> Self {
        Self {
            runtime: GameRuntime::from_snapshot(self.runtime.snapshot())
                .expect("TideLatch must restore its own runtime snapshot"),
            scene: self.scene.clone(),
        }
    }
}

impl fmt::Debug for TideLatchState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TideLatchState")
            .field("game", self.runtime.state())
            .finish_non_exhaustive()
    }
}

impl GlobalState for TideLatchState {}

#[fission_reducer(FrameTick)]
fn tick(state: &mut TideLatchState) {
    state.advance();
}
#[fission_reducer(Latch)]
fn latch(state: &mut TideLatchState) {
    state.dispatch(GameMessage::SetLatch(true));
}
#[fission_reducer(Release)]
fn release(state: &mut TideLatchState) {
    state.dispatch(GameMessage::SetLatch(false));
}
#[fission_reducer(Restart)]
fn restart(state: &mut TideLatchState) {
    state.dispatch(GameMessage::Restart);
}

fn host_input(
    state: &mut TideLatchState,
    action: GameHostInput,
    _ctx: &mut ReducerContext<TideLatchState>,
) {
    action.apply(&mut state.runtime);
    state.advance();
}

#[derive(Clone)]
pub struct TideLatchApp;

impl From<TideLatchApp> for Widget {
    fn from(_app: TideLatchApp) -> Self {
        let (ctx, view) = fission::build::current::<TideLatchState>();
        let tokens = &view.env().theme.tokens;
        let tick_action = with_reducer!(ctx, FrameTick, tick);
        let latch_action = with_reducer!(ctx, Latch, latch);
        let release_action = with_reducer!(ctx, Release, release);
        let restart_action = with_reducer!(ctx, Restart, restart);
        let input_action = ctx.bind(
            GameHostInput::FocusLost,
            host_input
                as fn(&mut TideLatchState, GameHostInput, &mut ReducerContext<TideLatchState>),
        );

        ctx.with_resources(|resources| {
            resources.timer(
                TimerResource::new(
                    ResourceKey::new("tidelatch-game-clock"),
                    Duration::from_millis(16),
                    FrameTick,
                )
                .on_tick(tick_action),
            );
        });

        let state = view.state();
        let game = state.runtime.state();
        let viewport = view.viewport_size();
        let available_width =
            (viewport.width - tokens.spacing.xl * 2.0).clamp(280.0, VIEWPORT_SIZE.x);
        let scene_height = (available_width * VIEWPORT_SIZE.y / VIEWPORT_SIZE.x)
            .min((viewport.height - 260.0).max(158.0));
        let scene_width = scene_height * VIEWPORT_SIZE.x / VIEWPORT_SIZE.y;
        let status_key = if game.finished() {
            "game.complete"
        } else if game.latched {
            "game.status_latched"
        } else if game.held_cargo > 0 {
            "game.status_loaded"
        } else {
            "game.status_ready"
        };

        let content = Container::new(Column {
            gap: Some(tokens.spacing.m),
            children: widgets![
                Row {
                    children: widgets![
                        Column {
                            gap: Some(tokens.spacing.xs),
                            children: widgets![
                                Text::new(TextContent::Key("game.title".into()))
                                    .size(tokens.typography.heading1_size)
                                    .color(tokens.colors.text_primary),
                                Text::new(TextContent::Key("game.subtitle".into()))
                                    .size(tokens.typography.body_medium_size)
                                    .color(tokens.colors.text_secondary),
                            ],
                            ..Default::default()
                        },
                        Spacer::default(),
                        Column {
                            gap: Some(tokens.spacing.xs),
                            children: widgets![
                                Text::new(format!("Score  {:04}", game.score))
                                    .size(tokens.typography.heading2_size)
                                    .color(tokens.colors.primary),
                                Text::new(format!(
                                    "Cargo  {} aboard · {} banked",
                                    game.held_cargo, game.banked_cargo
                                ))
                                .size(tokens.typography.font_size_sm)
                                .color(tokens.colors.text_secondary),
                            ],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                Text::new(TextContent::Key(status_key.into()))
                    .size(tokens.typography.body_medium_size)
                    .color(if game.latched {
                        tokens.colors.primary
                    } else {
                        tokens.colors.text_secondary
                    }),
                SemanticsRegion::new(
                    Container::new(
                        Scene2D::new(state.scene.clone())
                            .width(scene_width)
                            .height(scene_height)
                    )
                    .width(scene_width)
                    .height(scene_height)
                    .border(tokens.colors.border, 1.0)
                    .border_radius(tokens.radii.xl)
                    .clip_overflow(true),
                )
                .identifier("tidelatch.ocean"),
                Wrap {
                    direction: FlexDirection::Row,
                    spacing: Some(tokens.spacing.s),
                    run_spacing: Some(tokens.spacing.s),
                    children: widgets![
                        Button {
                            on_press: Some(latch_action),
                            child: Some(Text::new(TextContent::Key("game.latch".into())).into()),
                            ..Default::default()
                        }
                        .semantics_identifier("tidelatch.latch"),
                        Button {
                            on_press: Some(release_action),
                            child: Some(Text::new(TextContent::Key("game.release".into())).into()),
                            ..Default::default()
                        }
                        .semantics_identifier("tidelatch.release"),
                        Button {
                            on_press: Some(restart_action),
                            child: Some(Text::new(TextContent::Key("game.restart".into())).into()),
                            ..Default::default()
                        }
                        .semantics_identifier("tidelatch.restart"),
                        Text::new(TextContent::Key("game.controls".into()))
                            .size(tokens.typography.font_size_sm)
                            .color(tokens.colors.text_muted),
                    ],
                },
            ],
            ..Default::default()
        })
        .padding_all(tokens.spacing.xl)
        .bg(tokens.colors.background);

        GameInputRegion::for_game::<TideLatchGame>(content.into(), input_action)
            .semantics_identifier("tidelatch.game-input")
            .into()
    }
}

pub fn create_env() -> anyhow::Result<Env> {
    let mut env = Env::default();
    for (locale, yaml) in [
        ("en-US", include_str!("../i18n/en-US.yaml")),
        ("es-ES", include_str!("../i18n/es-ES.yaml")),
    ] {
        env.i18n.add_bundle(TranslationBundle {
            locale: Locale::from(locale),
            messages: serde_yaml::from_str::<HashMap<String, String>>(yaml)?,
        });
    }
    env.locale = Locale::from("en-US");
    Ok(env)
}
