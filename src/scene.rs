use fission::physics::Vec2;
use fission::scene2d::*;

use crate::game::{
    add, scale, sub, TideLatchGame, ANCHORS, CARGO, PLAYER_RADIUS, ROCKS, VIEWPORT_SIZE, WORLD_SIZE,
};

const SCENE_ID: SceneId = SceneId::new(1);
const TETHER_RESOURCE: ResourceId = ResourceId(1);

pub fn build_scene(game: &TideLatchGame) -> Scene2DIR {
    let mut scene = Scene2DIR::new(SCENE_ID, Viewport2D::new(VIEWPORT_SIZE.x, VIEWPORT_SIZE.y));
    scene.camera.center = scale(WORLD_SIZE, 0.5);
    scene.nodes.push(rectangle(
        1,
        Rect2D::new(Vec2::ZERO, WORLD_SIZE),
        Rgba::new(0.018, 0.105, 0.18, 1.0),
        0.0,
        0,
    ));
    for (index, y) in [76.0, 160.0, 255.0, 350.0, 445.0].into_iter().enumerate() {
        scene.nodes.push(rectangle(
            10 + index as u64,
            Rect2D::new(Vec2::new(0.0, y), Vec2::new(WORLD_SIZE.x, 30.0)),
            Rgba::new(0.03, 0.23 + index as f32 * 0.012, 0.34, 0.22),
            15.0,
            1,
        ));
    }
    add_harbor(&mut scene);

    for (index, anchor) in ANCHORS.iter().copied().enumerate() {
        let selected = game.active_anchor == Some(index) && game.latched;
        scene.nodes.push(circle(
            100 + index as u64,
            anchor,
            if selected { 19.0 } else { 13.0 },
            if selected {
                Rgba::new(0.98, 0.82, 0.31, 1.0)
            } else {
                Rgba::new(0.33, 0.73, 0.86, 1.0)
            },
            7,
        ));
        scene.nodes.push(circle(
            110 + index as u64,
            anchor,
            if selected { 7.0 } else { 5.0 },
            Rgba::new(0.02, 0.12, 0.2, 1.0),
            8,
        ));
    }
    if let Some(anchor) = game.active_anchor.filter(|_| game.latched) {
        let target = ANCHORS[anchor];
        scene.resources.paths.insert(
            TETHER_RESOURCE,
            PathResource2D {
                commands: vec![
                    PathCommand2D::MoveTo { point: game.player },
                    PathCommand2D::LineTo { point: target },
                ],
                bounds: Bounds2::new(
                    Vec2::new(game.player.x.min(target.x), game.player.y.min(target.y)),
                    Vec2::new(game.player.x.max(target.x), game.player.y.max(target.y)),
                ),
            },
        );
        scene.nodes.push(path(
            120,
            TETHER_RESOURCE,
            Rgba::new(0.98, 0.82, 0.31, 0.92),
            3.0,
            6,
        ));
    }

    for (index, (body, center, half_extents)) in ROCKS.iter().copied().enumerate() {
        let mut rock = rectangle(
            200 + body.get(),
            Rect2D::new(
                Vec2::new(-half_extents.x, -half_extents.y),
                scale(half_extents, 2.0),
            ),
            Rgba::new(0.09, 0.14, 0.19, 1.0),
            16.0,
            5,
        );
        rock.transform.translation = center;
        rock.transform.rotation_radians = if index % 2 == 0 { 0.12 } else { -0.16 };
        scene.nodes.push(rock);
    }
    for (index, cargo) in CARGO.iter().copied().enumerate() {
        if !game.collected[index] {
            scene.nodes.push(rectangle(
                300 + index as u64,
                Rect2D::new(sub(cargo, Vec2::new(10.0, 10.0)), Vec2::new(20.0, 20.0)),
                Rgba::new(0.96, 0.65, 0.19, 1.0),
                5.0,
                8,
            ));
        }
    }

    let speed = (game.velocity.x * game.velocity.x + game.velocity.y * game.velocity.y).sqrt();
    if speed > 12.0 {
        let direction = scale(game.velocity, 1.0 / speed);
        for index in 1..=3 {
            let point = sub(game.player, scale(direction, 18.0 + index as f32 * 15.0));
            scene.nodes.push(circle(
                400 + index,
                point,
                5.0 - index as f32 * 0.8,
                Rgba::new(0.58, 0.9, 0.96, 0.46),
                9,
            ));
        }
    }
    let trail_direction = if speed > 1.0 {
        scale(game.velocity, -1.0 / speed)
    } else {
        Vec2::new(-1.0, 0.0)
    };
    for index in 0..game.held_cargo {
        let center = add(
            game.player,
            scale(trail_direction, 34.0 + index as f32 * 23.0),
        );
        scene.nodes.push(rectangle(
            500 + index as u64,
            Rect2D::new(sub(center, Vec2::new(7.0, 7.0)), Vec2::new(14.0, 14.0)),
            Rgba::new(0.96, 0.65, 0.19, 1.0),
            4.0,
            10,
        ));
    }
    scene.nodes.push(boat(game));
    scene
}

fn add_harbor(scene: &mut Scene2DIR) {
    scene.nodes.push(circle(
        50,
        Vec2::new(92.0, 454.0),
        76.0,
        Rgba::new(0.05, 0.33, 0.38, 0.9),
        2,
    ));
    scene.nodes.push(rectangle(
        51,
        Rect2D::new(Vec2::new(18.0, 471.0), Vec2::new(149.0, 69.0)),
        Rgba::new(0.34, 0.25, 0.15, 1.0),
        12.0,
        3,
    ));
    scene.nodes.push(rectangle(
        52,
        Rect2D::new(Vec2::new(45.0, 431.0), Vec2::new(94.0, 15.0)),
        Rgba::new(0.76, 0.55, 0.26, 1.0),
        5.0,
        4,
    ));
}

fn boat(game: &TideLatchGame) -> Node2D {
    let speed = (game.velocity.x * game.velocity.x + game.velocity.y * game.velocity.y).sqrt();
    let rotation = if speed > 3.0 {
        game.velocity.y.atan2(game.velocity.x)
    } else {
        -0.2
    };
    Node2D {
        id: NodeId::new(600),
        parent: None,
        transform: Transform2 {
            translation: game.player,
            rotation_radians: rotation,
            ..Transform2::IDENTITY
        },
        visible: true,
        opacity: 1.0,
        layer: 12,
        blend_mode: BlendMode2D::Normal,
        clip: None,
        content: NodeContent2D::Rectangle {
            rect: Rect2D::new(
                Vec2::new(-PLAYER_RADIUS - 4.0, -PLAYER_RADIUS * 0.62),
                Vec2::new((PLAYER_RADIUS + 4.0) * 2.0, PLAYER_RADIUS * 1.24),
            ),
            style: PathStyle2D {
                fill: Some(Fill2D {
                    color: Rgba::new(0.93, 0.96, 0.93, 1.0),
                }),
                stroke: Some(Stroke2D {
                    color: Rgba::new(0.02, 0.12, 0.2, 1.0),
                    width: 3.0,
                }),
            },
            corner_radius: PLAYER_RADIUS,
        },
        interaction: None,
    }
}

fn rectangle(id: u64, rect: Rect2D, color: Rgba, radius: f32, layer: i32) -> Node2D {
    Node2D {
        id: NodeId::new(id),
        parent: None,
        transform: Transform2::IDENTITY,
        visible: true,
        opacity: 1.0,
        layer,
        blend_mode: BlendMode2D::Normal,
        clip: None,
        content: NodeContent2D::Rectangle {
            rect,
            style: PathStyle2D {
                fill: Some(Fill2D { color }),
                stroke: None,
            },
            corner_radius: radius,
        },
        interaction: None,
    }
}
fn circle(id: u64, center: Vec2, radius: f32, color: Rgba, layer: i32) -> Node2D {
    Node2D {
        id: NodeId::new(id),
        parent: None,
        transform: Transform2::IDENTITY,
        visible: true,
        opacity: 1.0,
        layer,
        blend_mode: BlendMode2D::Normal,
        clip: None,
        content: NodeContent2D::Rectangle {
            rect: Rect2D::new(
                sub(center, Vec2::new(radius, radius)),
                Vec2::new(radius * 2.0, radius * 2.0),
            ),
            style: PathStyle2D {
                fill: Some(Fill2D { color }),
                stroke: None,
            },
            corner_radius: radius,
        },
        interaction: None,
    }
}
fn path(id: u64, resource: ResourceId, color: Rgba, width: f32, layer: i32) -> Node2D {
    Node2D {
        id: NodeId::new(id),
        parent: None,
        transform: Transform2::IDENTITY,
        visible: true,
        opacity: 1.0,
        layer,
        blend_mode: BlendMode2D::Normal,
        clip: None,
        content: NodeContent2D::Path {
            path: resource,
            style: PathStyle2D {
                fill: None,
                stroke: Some(Stroke2D { color, width }),
            },
        },
        interaction: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_ocean_scene_prepares_without_diagnostics() {
        let scene = build_scene(&TideLatchGame::default());
        let prepared = Scene2DProcessor::prepare(&scene);
        assert!(
            prepared.diagnostics.is_empty(),
            "scene diagnostics: {:?}",
            prepared.diagnostics
        );
    }
}
