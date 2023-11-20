//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::{io::Read, str::Bytes};

use bevy::{
    asset::{io::Reader, meta::Settings, AssetLoader, AsyncReadExt},
    math::{vec2, vec3, quat},
    prelude::*,
};
use enemy::EnemyStates;
use player::PlayerStates;
use ron::{error::SpannedError, Map};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use util::*;

mod player;
mod enemy;
mod util;

#[derive(Component)]
struct HeartText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .add_systems(Update, player::update_player_movement)
        .add_systems(Update, enemy::update_enemy_movement)
        .run();
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let betty_handle = asset_server.load("sprites/atlases/betty_mercy.png");
    let mut betty_atlas = TextureAtlas::new_empty(betty_handle, vec2(481.0, 178.0));
    let abigail_handle = asset_server.load("sprites/atlases/abigail.png");
    let mut abigail_atlas = TextureAtlas::new_empty(abigail_handle, vec2(1118.0, 211.0));

    //Betty

    betty_atlas.add_texture(AtlasUtil::from_corner_size(1., 1., 24., 88.)); // Idle
    betty_atlas.add_texture(AtlasUtil::from_corner_size(26., 1., 24., 88.));

    betty_atlas.add_texture(AtlasUtil::from_corner_size(126., 1., 32., 88.)); // Punch
    betty_atlas.add_texture(AtlasUtil::from_corner_size(159., 1., 32., 88.));
    betty_atlas.add_texture(AtlasUtil::from_corner_size(192., 1., 24., 88.));
    betty_atlas.add_texture(AtlasUtil::from_corner_size(217., 1., 24., 88.));

    betty_atlas.add_texture(AtlasUtil::from_corner_size(151., 90., 32., 88.)); // Hit
    betty_atlas.add_texture(AtlasUtil::from_corner_size(184., 90., 32., 88.));

    betty_atlas.add_texture(AtlasUtil::from_corner_size(349., 90., 24., 88.)); // Block
    betty_atlas.add_texture(AtlasUtil::from_corner_size(374., 90., 24., 88.));

    betty_atlas.add_texture(AtlasUtil::from_corner_size(51., 90., 24., 88.)); // Dodge
    betty_atlas.add_texture(AtlasUtil::from_corner_size(76., 90., 24., 88.));

    //Abigail

    abigail_atlas.add_texture(AtlasUtil::from_corner_size(1., 1., 40., 104.)); // Idle
    abigail_atlas.add_texture(AtlasUtil::from_corner_size(42., 1., 40., 104.)); 

    abigail_atlas.add_texture(AtlasUtil::from_corner_size(329., 1., 32., 104.)); // Punch
    abigail_atlas.add_texture(AtlasUtil::from_corner_size(370., 1., 32., 104.));
    abigail_atlas.add_texture(AtlasUtil::from_corner_size(1., 106., 40., 104.));

    abigail_atlas.add_texture(AtlasUtil::from_corner_size(272., 106., 40., 104.)); // Hit

    abigail_atlas.add_texture(AtlasUtil::from_corner_size(91., 106., 32., 104.)); // Block



    let betty_atlas_handle = texture_atlases.add(betty_atlas);
    let abigail_atlas_handle = texture_atlases.add(abigail_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 1 };
    let betty_sprite = TextureAtlasSprite::new(animation_indices.first);
    let abigail_sprite = TextureAtlasSprite::new(0);
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: betty_atlas_handle,
            sprite: betty_sprite,
            transform: Transform::from_scale(Vec3::splat(2.0)),
            ..default()
        },
        player::Player::new(
            Vec3 {
                x: 0.0,
                y: -80.0,
                z: 1.0,
            },
            Animator::new(
                AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)),
                AnimationIndices { first: 0, last: 1 },
                true,
            ),
            PlayerStates {
                idle: AnimationIndices { first: 0, last: 1 },
                punch: AnimationIndices { first: 2, last: 5 },
                hit: AnimationIndices { first: 6, last: 7 },
                block: AnimationIndices { first: 8, last: 9 },
                dodge: AnimationIndices { first: 10, last: 11 },
            },
        )
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/stage.png"),
            transform: Transform { translation: vec3(0.0, 0.0, -1.0), rotation: quat(0.0, 0.0, 0.0, 1.0), scale: Vec3::splat(2.0) },
            ..default()
        }
    ));
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: abigail_atlas_handle,
            sprite: abigail_sprite,
            transform: Transform { translation: vec3(0.0, 0.0, 0.0), rotation: quat(0.0, 0.0, 0.0, 1.0), scale: Vec3::splat(2.0) },
            ..default()
        },
        enemy::Enemy::new(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Animator::new(
                AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)),
                AnimationIndices { first: 0, last: 1 },
                true,
            ),
            EnemyStates {
                idle: AnimationIndices { first: 0, last: 1 },
                punch_warning: AnimationIndices { first: 2, last: 3},
                punch: AnimationIndices { first: 4, last: 4 },
                hit: AnimationIndices { first: 5, last: 5 },
                block: AnimationIndices { first: 6, last: 6 },
            },
        )
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "00",
            TextStyle { font: asset_server.load("fonts/font.ttf"), font_size: 16.0, color: Color::WHITE }
            ).with_alignment(TextAlignment::Right),
            transform: Transform::from_translation(Vec3 { x: -128.0, y: 182.0, z: 0.0 }),
            ..default()
        }
    ));
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "00",
            TextStyle { font: asset_server.load("fonts/font.ttf"), font_size: 16.0, color: Color::WHITE }
            ).with_alignment(TextAlignment::Right),
            transform: Transform::from_translation(Vec3 { x: -126.0, y: 182.0, z: 0.0 }),
            ..default()
        }
    ));
}
