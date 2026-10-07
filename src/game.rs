use fission::game::{
    Game, GameCtx, GameKey, GameState, GameTime, InputMap, InputTrigger, StepCtx, StepDuration,
};
use fission::physics::{
    Collider2D, PhysicsBody2D, PhysicsBodyId, PhysicsPose2D, PhysicsProvider2D, PhysicsShape2D,
    PhysicsSnapshot2D, PhysicsVelocity2D, Vec2,
};
use fission::physics_rapier2d::RapierPhysicsWorld2D;
use fission::scene2d::Scene2DIR;
use serde::{Deserialize, Serialize};

use crate::scene::build_scene;

pub const VIEWPORT_SIZE: Vec2 = Vec2::new(960.0, 540.0);
pub const WORLD_SIZE: Vec2 = VIEWPORT_SIZE;
pub const PLAYER_RADIUS: f32 = 18.0;
pub const PLAYER_BODY: PhysicsBodyId = PhysicsBodyId::new(1);
const START: Vec2 = Vec2::new(126.0, 420.0);
const HARBOR: Vec2 = Vec2::new(92.0, 454.0);
const HARBOR_RADIUS: f32 = 74.0;
const LATCH_ACCELERATION: f32 = 360.0;
const MAX_SPEED: f32 = 330.0;

pub const ANCHORS: [Vec2; 4] = [
    Vec2::new(255.0, 126.0),
    Vec2::new(492.0, 400.0),
    Vec2::new(724.0, 122.0),
    Vec2::new(842.0, 414.0),
];
pub const CARGO: [Vec2; 6] = [
    Vec2::new(334.0, 225.0),
    Vec2::new(438.0, 92.0),
    Vec2::new(574.0, 454.0),
    Vec2::new(678.0, 315.0),
    Vec2::new(816.0, 230.0),
    Vec2::new(894.0, 345.0),
];
pub const ROCKS: [(PhysicsBodyId, Vec2, Vec2); 3] = [
    (
        PhysicsBodyId::new(10),
        Vec2::new(400.0, 318.0),
        Vec2::new(46.0, 34.0),
    ),
    (
        PhysicsBodyId::new(11),
        Vec2::new(607.0, 228.0),
        Vec2::new(42.0, 50.0),
    ),
    (
        PhysicsBodyId::new(12),
        Vec2::new(780.0, 374.0),
        Vec2::new(38.0, 30.0),
    ),
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TideLatchGame {
    pub player: Vec2,
    pub velocity: Vec2,
    pub latched: bool,
    pub active_anchor: Option<usize>,
    pub collected: [bool; CARGO.len()],
    pub held_cargo: u32,
    pub banked_cargo: u32,
    pub score: u32,
    pub voyage: u32,
    pub physics: PhysicsSnapshot2D,
    pub last_error: Option<String>,
    carried: Vec<usize>,
    collision_cooldown: u16,
}

impl Default for TideLatchGame {
    fn default() -> Self {
        Self::new().expect("the built-in TideLatch physics arena is valid")
    }
}

impl TideLatchGame {
    fn new() -> Result<Self, String> {
        let mut world = RapierPhysicsWorld2D::new(Vec2::ZERO).map_err(|e| e.to_string())?;
        let mut player = PhysicsBody2D::dynamic(
            PLAYER_BODY,
            PhysicsShape2D::Circle {
                radius: PLAYER_RADIUS,
            },
        );
        player.pose = PhysicsPose2D::new(START, 0.0);
        player.gravity_scale = 0.0;
        player.linear_damping = 0.32;
        player.continuous_collision_detection = true;
        player.colliders[0].restitution = 0.82;
        player.colliders[0].friction = 0.05;
        world.insert_body(player).map_err(|e| e.to_string())?;
        for (id, center, half_extents) in ROCKS {
            insert_box(&mut world, id, center, half_extents)?;
        }
        for (id, center, half_extents) in [
            (
                20,
                Vec2::new(WORLD_SIZE.x / 2.0, -12.0),
                Vec2::new(WORLD_SIZE.x / 2.0, 12.0),
            ),
            (
                21,
                Vec2::new(WORLD_SIZE.x / 2.0, WORLD_SIZE.y + 12.0),
                Vec2::new(WORLD_SIZE.x / 2.0, 12.0),
            ),
            (
                22,
                Vec2::new(-12.0, WORLD_SIZE.y / 2.0),
                Vec2::new(12.0, WORLD_SIZE.y / 2.0),
            ),
            (
                23,
                Vec2::new(WORLD_SIZE.x + 12.0, WORLD_SIZE.y / 2.0),
                Vec2::new(12.0, WORLD_SIZE.y / 2.0),
            ),
        ] {
            insert_box(&mut world, PhysicsBodyId::new(id), center, half_extents)?;
        }

        Ok(Self {
            player: START,
            velocity: Vec2::ZERO,
            latched: false,
            active_anchor: None,
            collected: [false; CARGO.len()],
            held_cargo: 0,
            banked_cargo: 0,
            score: 0,
            voyage: 1,
            physics: world.snapshot(),
            last_error: None,
            carried: Vec::new(),
            collision_cooldown: 0,
        })
    }

    pub fn finished(&self) -> bool {
        self.banked_cargo == CARGO.len() as u32
    }

    fn set_latch(&mut self, active: bool) {
        self.latched = active;
        self.active_anchor = active.then(|| nearest_anchor(self.player));
    }

    fn simulate(&mut self, duration: StepDuration) -> Result<(), String> {
        let mut world = RapierPhysicsWorld2D::new(Vec2::ZERO).map_err(|e| e.to_string())?;
        world
            .restore(self.physics.clone())
            .map_err(|e| e.to_string())?;
        let mut velocity = world
            .body_velocity(PLAYER_BODY)
            .unwrap_or_else(|| PhysicsVelocity2D::new(self.velocity, 0.0));
        if let Some(anchor) = self.active_anchor.filter(|_| self.latched) {
            let pull = normalized(sub(ANCHORS[anchor], self.player));
            velocity.linear = clamp_length(
                add(
                    velocity.linear,
                    scale(pull, LATCH_ACCELERATION * duration.as_secs_f32()),
                ),
                MAX_SPEED,
            );
        }
        world
            .set_body_velocity(PLAYER_BODY, velocity, true)
            .map_err(|e| e.to_string())?;
        world.step(duration);
        if let Some(pose) = world.body_pose(PLAYER_BODY) {
            self.player = pose.translation;
        }
        if let Some(value) = world.body_velocity(PLAYER_BODY) {
            self.velocity = value.linear;
        }

        if self.collision_cooldown > 0 {
            self.collision_cooldown -= 1;
        } else if self.held_cargo > 0
            && world.contacts().iter().any(|c| {
                (c.first == PLAYER_BODY && is_rock(c.second))
                    || (c.second == PLAYER_BODY && is_rock(c.first))
            })
        {
            self.held_cargo -= 1;
            if let Some(index) = self.carried.pop() {
                self.collected[index] = false;
            }
            self.collision_cooldown = 30;
        }
        for (index, cargo) in CARGO.iter().copied().enumerate() {
            if !self.collected[index] && distance(self.player, cargo) <= PLAYER_RADIUS + 19.0 {
                self.collected[index] = true;
                self.carried.push(index);
                self.held_cargo += 1;
                self.score = self.score.saturating_add(25);
            }
        }
        if self.held_cargo > 0 && distance(self.player, HARBOR) <= HARBOR_RADIUS {
            let delivery = self.held_cargo;
            self.score = self
                .score
                .saturating_add(100 * delivery + 40 * delivery.saturating_sub(1));
            self.banked_cargo += delivery;
            self.held_cargo = 0;
            self.carried.clear();
        }
        self.physics = world.snapshot();
        Ok(())
    }
}

impl GameState for TideLatchGame {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMessage {
    SetLatch(bool),
    Restart,
}

impl Game for TideLatchGame {
    type Message = GameMessage;
    type Presentation = Scene2DIR;

    fn input(input: &mut InputMap<Self::Message>) {
        input
            .on(InputTrigger::KeyPressed {
                key: GameKey::Space,
            })
            .send(GameMessage::SetLatch(true));
        input
            .on(InputTrigger::KeyReleased {
                key: GameKey::Space,
            })
            .send(GameMessage::SetLatch(false));
    }
    fn react(&mut self, message: Self::Message, _ctx: &mut GameCtx<'_, Self>) {
        match message {
            GameMessage::SetLatch(active) => self.set_latch(active),
            GameMessage::Restart => *self = Self::default(),
        }
    }
    fn step(&mut self, ctx: &mut StepCtx<'_, Self>) {
        if let Err(error) = self.simulate(ctx.duration()) {
            self.last_error = Some(error);
        }
    }
    fn present(&self, _time: GameTime) -> Self::Presentation {
        build_scene(self)
    }
}

fn insert_box(
    world: &mut RapierPhysicsWorld2D,
    id: PhysicsBodyId,
    center: Vec2,
    half_extents: Vec2,
) -> Result<(), String> {
    let shape = PhysicsShape2D::Cuboid { half_extents };
    let mut body = PhysicsBody2D::fixed(id, shape.clone());
    body.pose = PhysicsPose2D::new(center, 0.0);
    body.colliders[0] = Collider2D::new(shape);
    body.colliders[0].restitution = 0.7;
    body.colliders[0].friction = 0.08;
    world.insert_body(body).map_err(|e| e.to_string())
}

fn nearest_anchor(position: Vec2) -> usize {
    ANCHORS
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| distance(position, **a).total_cmp(&distance(position, **b)))
        .map(|(index, _)| index)
        .unwrap_or(0)
}
fn is_rock(id: PhysicsBodyId) -> bool {
    ROCKS.iter().any(|(rock, _, _)| *rock == id)
}
pub fn add(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x + b.x, a.y + b.y)
}
pub fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x - b.x, a.y - b.y)
}
pub fn scale(value: Vec2, factor: f32) -> Vec2 {
    Vec2::new(value.x * factor, value.y * factor)
}
pub fn distance(a: Vec2, b: Vec2) -> f32 {
    let d = sub(a, b);
    (d.x * d.x + d.y * d.y).sqrt()
}
fn normalized(value: Vec2) -> Vec2 {
    let len = (value.x * value.x + value.y * value.y).sqrt();
    if len <= f32::EPSILON {
        Vec2::ZERO
    } else {
        scale(value, 1.0 / len)
    }
}
fn clamp_length(value: Vec2, maximum: f32) -> Vec2 {
    let len = (value.x * value.x + value.y * value.y).sqrt();
    if len > maximum {
        scale(value, maximum / len)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use fission::game::GameRuntime;

    use super::*;

    #[test]
    fn holding_the_latch_builds_momentum_toward_an_anchor() {
        let game = TideLatchGame::default();
        let start = game.player;
        let mut runtime = GameRuntime::new(game);
        runtime.send(GameMessage::SetLatch(true));
        for _ in 0..60 {
            runtime.advance(StepDuration::from_hz(60).as_duration());
        }

        assert!(distance(runtime.state().player, start) > 20.0);
        assert!(runtime.state().latched);
        assert!(runtime.state().active_anchor.is_some());
    }

    #[test]
    fn restarting_restores_the_initial_voyage() {
        let mut runtime = GameRuntime::new(TideLatchGame::default());
        runtime.send(GameMessage::SetLatch(true));
        runtime.advance(StepDuration::from_hz(60).as_duration());
        runtime.send(GameMessage::Restart);
        runtime.advance(StepDuration::from_hz(60).as_duration());

        assert_eq!(runtime.state().player, START);
        assert!(!runtime.state().latched);
        assert_eq!(runtime.state().score, 0);
    }
}
