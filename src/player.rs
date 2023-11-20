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

use crate::{util::*, AnimationIndices, AnimationTimer, enemy::{Enemy, self}};

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
    pub punch_timer: f32,
    states: PlayerStates,
    blocking: bool,
    origin: Vec3,
    animator: Animator,
    is_hit_timer: f32,
    hits_taken_total: i32,
}

pub struct PlayerStates {
    pub idle: AnimationIndices,
    pub punch: AnimationIndices,
    pub hit: AnimationIndices,
    pub block: AnimationIndices,
    pub dodge: AnimationIndices,
    pub death: AnimationIndices
}

impl Player {
    pub fn new(origin: Vec3, animator: Animator, states: PlayerStates) -> Player {
        Player {
            states: states,
            action_dir: ActionDirection::None,
            punch_timer: 0.0,
            dodge_timer: 0.0,
            is_hit_timer: 0.0,
            blocking: false,
            origin: origin,
            animator: animator,
            hits_taken_total: 0,
        }
    }
}

const DODGE_DISTANCE: f32 = 75.0;
const DODGE_DURATION: f32 = 0.75;
pub const PUNCH_DURATION: f32 = 0.5;
const IS_HIT_TIMER: f32 = 0.1;

pub fn update_player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Player, &mut Transform, &mut TextureAtlasSprite)>,
    enemy_query: Query<(&mut Enemy)>
) {
    'player_loop: for (mut player, mut transform, mut sprite) in &mut player_query {
        for enemy in &enemy_query {
            if player.hits_taken_total > 15 {
                let state = player.states.death;
                player.animator.set_indices(state);
                player.animator.set_frametime(0.1);
                player.animator.loops = false;
                break 'player_loop;
            }
            if enemy.punch_timer < enemy::PUNCH_DURATION && enemy.punch_timer >= 0. {
                if !player.blocking && player.dodge_timer == 0. {
                    let state = player.states.hit;
                    player.animator.set_indices(state);
                    player.animator.set_frametime(0.1);
                    player.animator.loops = false;
                    player.is_hit_timer = IS_HIT_TIMER;
                    player.punch_timer = 0.;
                    player.hits_taken_total += 1;

                    sprite.index = player.animator.index;
                    player.animator.tick(time.delta()); 
                    continue 'player_loop;
                }
            }

            // not dodging
            match (player.dodge_timer, player.punch_timer, player.is_hit_timer) {
                (dodge_timer, punch_timer, is_hit_timer) if dodge_timer <= 0. && punch_timer <= 0. && is_hit_timer <= 0. => {
                    // put player at origin
                    transform.translation = player.origin;
                    player.blocking = false;

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

                            player.blocking = true;
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
                (_, _, is_hit_timer) if is_hit_timer > 0. => {
                    if player.is_hit_timer < IS_HIT_TIMER {
                        let state = player.states.hit;
                        player.animator.set_indices(state);
                        player.animator.set_frametime(0.3);
                    }
                    player.is_hit_timer -= time.delta_seconds()
                }
                (dodge_timer, punch_timer, _) if dodge_timer > 0. && punch_timer <= 0. => {
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
                (_, punch_timer, _) if punch_timer > 0. => {
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
}
