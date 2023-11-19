use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    input::{keyboard::KeyCode, Input},
    math::{vec2, Vec3},
    prelude::default,
    time::Time,
    transform::components::Transform, sprite::{SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
};

use crate::{util::*, AnimationIndices, AnimationTimer};

#[derive(Default)]
enum ActionDirection {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Component)]
pub struct Player {
    dodge_timer: f32,
    action_dir: ActionDirection,
    punch_timer: f32,
    states: PlayerStates,
    blocking: bool,
    origin: Vec3,
    animator: Animator,
}

pub struct PlayerStates {
    pub idle: AnimationIndices,
    pub punch: AnimationIndices,
    pub hit: AnimationIndices,
    pub block: AnimationIndices,
    pub dodge: AnimationIndices,
}

impl Player {
    pub fn new(origin: Vec3, animator: Animator, states: PlayerStates) -> Player {
        Player {
            states: states,
            action_dir: ActionDirection::None,
            punch_timer: 0.0,
            dodge_timer: 0.0,
            blocking: false,
            origin: origin,
            animator: animator,
        }
    }
}

const DODGE_DISTANCE: f32 = 75.0;
const DODGE_DURATION: f32 = 0.75;
const PUNCH_DURATION: f32 = 0.5;

pub fn update_player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut transform, mut sprite) in &mut query {
        // not dodging
        match (player.dodge_timer, player.punch_timer) {
            (dodge_timer, punch_timer) if dodge_timer <= 0. && punch_timer <= 0. => {
                // put player at origin
                transform.translation = player.origin;

                let (blocking, left_dodge, right_dodge, left_punch, right_punch) = (
                    input.pressed(KeyCode::Down),
                    input.just_pressed(KeyCode::Left),
                    input.just_pressed(KeyCode::Right),
                    input.just_pressed(KeyCode::Z),
                    input.just_pressed(KeyCode::X)
                );

                match (blocking, left_dodge, right_dodge, left_punch, right_punch) {
                    // blocking
                    (true, _, _, _, _) => {
                        let state = player.states.block;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.1);
                        player.animator.loops = false;
                    },
                    // left move
                    (_, true, _, _, _) => {
                        sprite.flip_x = false;
                        let state = player.states.dodge;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.1);
                        player.animator.loops = false;
                        player.action_dir = ActionDirection::Left;
                        player.dodge_timer = DODGE_DURATION;
                    },
                    // right move
                    (_, _, true, _, _) => {
                        sprite.flip_x = true;
                        let state = player.states.dodge;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.1);
                        player.animator.loops = false;
                        player.action_dir = ActionDirection::Right;
                        player.dodge_timer = DODGE_DURATION;
                    }
                    // left punch
                    (_, _, _, true, _) => {
                        sprite.flip_x = false;
                        let state = player.states.punch;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.03);
                        player.animator.loops = false;
                        player.action_dir = ActionDirection::Left;
                        player.punch_timer = PUNCH_DURATION;
                    },
                    // right punch
                    (_, _, _, _, true) => {
                        sprite.flip_x = true;
                        let state = player.states.punch;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.03);
                        player.animator.loops = false;
                        player.action_dir = ActionDirection::Right;
                        player.punch_timer = PUNCH_DURATION;
                    },
                    // nothing
                    _ => {
                        let state = player.states.idle;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.3);
                        player.animator.loops = true;
                        player.dodge_timer = 0.0;
                    }
                }
                
            },
            (dodge_timer, punch_timer) if dodge_timer > 0. && punch_timer <= 0. => {
                player.blocking = false;
                let x = (1.0 - (player.dodge_timer / DODGE_DURATION)) * 2.0;
                let x = x - 1.0;
                let x = x*x*x*x;
                transform.translation = player.origin
                    + Vec3::new(
                        match player.action_dir {
                            ActionDirection::None => 0.0,
                            ActionDirection::Left => x - 1.0,
                            ActionDirection::Right => -x + 1.0,
                        } * DODGE_DISTANCE,
                        0.0,
                        0.0,
                    );
                player.dodge_timer -= time.delta_seconds()
            },
            (_, punch_timer) if punch_timer > 0. => {
                let x = (1.0 - (player.punch_timer / PUNCH_DURATION)) * 2.0;
                let x = x - 1.0;
                let x = x*x*x*x;
                transform.translation = player.origin
                    + Vec3::new(
                        match player.action_dir {
                            ActionDirection::None => 0.0,
                            ActionDirection::Left => x - 1.0,
                            ActionDirection::Right => -x + 1.0,
                        } * -5.0,
                        -x * 15.0,
                        0.0,
                    );
                player.punch_timer -= time.delta_seconds()
            },
            _ => {}
        }
        sprite.index = player.animator.index;
        player.animator.tick(time.delta());
    }
}
