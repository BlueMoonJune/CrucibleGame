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

use crate::{util::*, AnimationIndices, AnimationTimer, player::{Player, self}};

use rand::*;
use rand::prelude::*;

#[derive(Default)]
enum ActionDirection {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Component)]
pub struct Enemy {
    action_dir: ActionDirection,
    pub punch_timer: f32,
    states: EnemyStates,
    block_timer: f32,
    wait_timer: f32,
    origin: Vec3,
    animator: Animator,
    is_hit_timer: f32,
    hits_taken: i32,
    hits_taken_total: i32,
}

pub struct EnemyStates {
    pub idle: AnimationIndices,
    pub punch_warning: AnimationIndices,
    pub punch: AnimationIndices,
    pub hit: AnimationIndices,
    pub block: AnimationIndices,
}

impl Enemy {
    pub fn new(origin: Vec3, animator: Animator, states: EnemyStates) -> Enemy {
        Enemy {
            states: states,
            action_dir: ActionDirection::None,
            punch_timer: 0.0,
            block_timer: 0.0,
            is_hit_timer: 0.0,
            wait_timer: 0.0,
            origin: origin,
            animator: animator,
            hits_taken: 0,
            hits_taken_total: 0,
        }
    }
}

const BLOCK_DURATION: f32 = 0.75;
const PUNCH_WARNING_DURATION: f32 = 1.0;
pub const PUNCH_DURATION: f32 = 0.5;
pub const IS_HIT_TIMER: f32 = 0.05;

pub fn update_enemy_movement(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Enemy, &mut Transform, &mut TextureAtlasSprite)>,
    player_query: Query<(&mut Player)>,
) {
    let mut rng = rand::thread_rng();
    'enemy_loop: for (mut enemy, mut transform, mut sprite) in &mut enemy_query {

        for player in &player_query {
            if player::PUNCH_DURATION - player.punch_timer < player::PUNCH_DURATION / 4. && player.punch_timer >= 0. && enemy.hits_taken < 2 {
                if enemy.block_timer == 0. {
                    let state = enemy.states.hit;
                    enemy.animator.set_indices(state);
                    enemy.animator.set_frametime(0.1);
                    enemy.animator.loops = false;
                    enemy.is_hit_timer = IS_HIT_TIMER;
                    enemy.punch_timer = PUNCH_WARNING_DURATION + PUNCH_DURATION;
                    enemy.hits_taken += 1;
                    enemy.hits_taken_total += 1;

                    sprite.index = enemy.animator.index;
                    enemy.animator.tick(time.delta()); 
                    continue 'enemy_loop;
                }
            }

            match (enemy.block_timer, enemy.punch_timer, enemy.wait_timer, enemy.is_hit_timer) {

                (block_timer, punch_timer, wait_timer, is_hit_timer) if block_timer <= 0. && punch_timer <= 0. && wait_timer <= 0. && is_hit_timer <= 0. => {
                    transform.translation = enemy.origin;
    
                    let mut action: u8 = rng.gen();
                    action %= 6;
    
                    match action {
                        // block
                        1 => {
                            let state = enemy.states.block;
                            enemy.animator.set_indices(state);
                            enemy.animator.set_frametime(0.1);
                            enemy.animator.loops = false;
                            enemy.block_timer = BLOCK_DURATION;
                        },
                        // left punch
                        2 => {
                            sprite.flip_x = false;
                            let state = enemy.states.punch_warning;
                            enemy.animator.set_indices(state);
                            enemy.animator.set_frametime(0.2);
                            enemy.animator.loops = true;
                            enemy.action_dir = ActionDirection::Left;
                            enemy.punch_timer = PUNCH_WARNING_DURATION + PUNCH_DURATION;
                        },
                        // right punch
                        3 => {
                            sprite.flip_x = true;
                            let state = enemy.states.punch_warning;
                            enemy.animator.set_indices(state);
                            enemy.animator.set_frametime(0.2);
                            enemy.animator.loops = true;
                            enemy.action_dir = ActionDirection::Right;
                            enemy.punch_timer = PUNCH_WARNING_DURATION + PUNCH_DURATION;
                        },
                        // nothing
                        _ => {
                            let state = enemy.states.idle;
                            enemy.animator.set_indices(state);
                            enemy.animator.set_frametime(0.3);
                            enemy.animator.loops = true;
                            enemy.block_timer = 0.0;
                            enemy.wait_timer = 1.5;
                        },
                    }
    
                },
                (_, _, _, is_hit_timer) if is_hit_timer > 0. => {
                    if enemy.is_hit_timer < IS_HIT_TIMER {
                        let state = enemy.states.hit;
                        enemy.animator.set_indices(state);
                        enemy.animator.set_frametime(0.3);
                    }
                    enemy.is_hit_timer -= time.delta_seconds()
                },
                (block_timer, punch_timer, _, is_hit_timer) if block_timer > 0. && punch_timer <= 0. && is_hit_timer <= 0. => {
                    enemy.hits_taken = 0;
                    enemy.is_hit_timer = 0.;

                    enemy.block_timer -= time.delta_seconds()
                },
                (_, punch_timer, _, is_hit_timer) if punch_timer > 0. && is_hit_timer <= 0. => {

                    if enemy.punch_timer < PUNCH_DURATION {
                        enemy.hits_taken = 0;
                        enemy.is_hit_timer = 0.;

                        let state = enemy.states.punch;
                        enemy.animator.set_indices(state);
                        enemy.animator.set_frametime(0.3);
                        let x = (enemy.punch_timer / PUNCH_DURATION) * 2.0 - 1.0;
                        let x = -x*x*x*x + 1.0;
                        transform.translation = enemy.origin
                            + Vec3::new(
                                match enemy.action_dir {
                                    ActionDirection::None => 0.0,
                                    ActionDirection::Left => x - 1.0,
                                    ActionDirection::Right => -x + 1.0,
                                } * 5.0,
                                x * -60.0,
                                0.0,
                            );
                    }
    
                    enemy.punch_timer -= time.delta_seconds();
                },
                (_, _, wait_timer, is_hit_timer) if wait_timer > 0. && is_hit_timer <= 0. => {
                    enemy.hits_taken = 0;
                    enemy.is_hit_timer = 0.;

                    enemy.wait_timer -= time.delta_seconds()
                }
                _ => {}
                
            }
            sprite.index = enemy.animator.index;
            enemy.animator.tick(time.delta());
        }
    }
}