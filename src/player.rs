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
enum DodgeDirection {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Component)]
pub struct Player {
    dodge_timer: f32,
    dodge_dir: DodgeDirection,
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
            dodge_dir: DodgeDirection::None,
            dodge_timer: 0.0,
            blocking: false,
            origin: origin,
            animator: animator,
        }
    }
}

const DODGE_DISTANCE: f32 = 75.0;
const DODGE_DURATION: f32 = 0.75;

pub fn update_player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut transform, mut sprite) in &mut query {
        if player.dodge_timer <= 0.0 {

            player.blocking = input.pressed(KeyCode::Down);

            if player.blocking {
                let state = player.states.block;
                player.animator.set_indices(state);
                player.animator.set_frametime(0.1);
                player.animator.loops = false;
        
            } else {
                let state = player.states.idle;
                player.animator.set_indices(state);
                player.animator.set_frametime(0.3);
                player.animator.loops = true;
                player.dodge_timer = 0.0;
                if input.just_pressed(KeyCode::Left) {
                    sprite.flip_x = false;
                    let state = player.states.dodge;
                    player.animator.set_indices(state);
                    player.animator.set_frametime(0.1);
                    player.animator.loops = false;
                    player.dodge_dir = DodgeDirection::Left;
                    player.dodge_timer = DODGE_DURATION;
                }

                if input.just_pressed(KeyCode::Right) {
                    sprite.flip_x = true;
                    let state = player.states.dodge;
                    player.animator.set_indices(state);
                    player.animator.set_frametime(0.1);
                    player.animator.loops = false;
                    player.dodge_dir = DodgeDirection::Right;
                    player.dodge_timer = DODGE_DURATION;
                }
            }

        } else {
            player.blocking = false;
            let x = (1.0 - (player.dodge_timer / DODGE_DURATION)) * 2.0;
            let x = x - 1.0;
            let x = x*x*x*x;
            transform.translation = player.origin
                + Vec3::new(
                    match player.dodge_dir {
                        DodgeDirection::None => 0.0,
                        DodgeDirection::Left => x - 1.0,
                        DodgeDirection::Right => -x + 1.0,
                    } * DODGE_DISTANCE,
                    0.0,
                    0.0,
                );
            player.dodge_timer -= time.delta_seconds()
        }
        sprite.index = player.animator.index;
        player.animator.tick(time.delta());
    }
}
