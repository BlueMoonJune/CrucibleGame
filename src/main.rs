/*
use bevy::{prelude::*, transform::commands, reflect::Enum};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, idle_movement)
        .run();
}

#[derive(Component)]
struct SpriteState {
    atlas: TextureAtlas,
}

#[derive(Component)]
struct SpriteStateTest {

}

impl SpriteStateTest {
    
}


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    // Background
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/atlases/stages.png"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
    ));
    // Abby
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/atlases/abigail.png"),
            transform: Transform {
                translation: Vec3::new(0., 225., 1.),
                rotation: Quat::from_euler(EulerRot::ZYX, 0., 0., 0.),
                scale: Vec3::new(4., 4., 4.),
            },
            ..default()
        },
        IdleInfo {
            direction: (IdleDirection::Vertical, 1),
            bounds: (200., 250.),
            speed: 300.,
        },
    ));
    // Betty
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/atlases/betty_mercy.png"),
            transform: Transform {
                translation: Vec3::new(0., -100., 2.),
                rotation: Quat::from_euler(EulerRot::ZYX, 0., 0., 0.),
                scale: Vec3::new(4., 4., 4.),
            },
            ..default()
        },
        IdleInfo {
            direction: (IdleDirection::Horizontal, 1),
            bounds: (-30., 30.),
            speed: 150.,
        },
    ));
}


/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn idle_movement(time: Res<Time>, mut sprite_position: Query<(&mut IdleInfo, &mut Transform)>) {
    for (mut anim_info, mut transform) in &mut sprite_position {
        match anim_info.direction.0 {
            IdleDirection::Vertical => {
                transform.translation.y += anim_info.speed * (anim_info.direction.1 as f32) * time.delta_seconds();

                if transform.translation.y < anim_info.bounds.0 {
                    transform.translation.y = anim_info.bounds.0;
                    anim_info.direction.1 *= -1;
                } else if transform.translation.y > anim_info.bounds.1 {
                    transform.translation.y = anim_info.bounds.1;
                    anim_info.direction.1 *= -1;
                }
            },
            IdleDirection::Horizontal => {
                transform.translation.x += anim_info.speed * (anim_info.direction.1 as f32) * time.delta_seconds();

                if transform.translation.x < anim_info.bounds.0 {
                    transform.translation.x = anim_info.bounds.0;
                    anim_info.direction.1 *= -1;
                } else if transform.translation.x > anim_info.bounds.1 {
                    transform.translation.x = anim_info.bounds.1;
                    anim_info.direction.1 *= -1;
                }
            },
        }
    }
}
*/

//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::{io::Read, str::Bytes};

use bevy::{prelude::*, asset::{AssetLoader, meta::Settings, io::Reader, AsyncReadExt}};
use ron::{Map, error::SpannedError};
use serde::{Serialize, Deserializer, de::{Visitor, self}, Deserialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .register_asset_loader(SpriteFramesLoader)
        .init_asset_loader::<SpriteFramesLoader>()
        .run();
}

/*
struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: de::Error, {
        Ok(String::from(value))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string.")
    }
}
*/

#[derive(Deserialize)]
struct SpriteState<> {
    frames: Vec<Rect>,
    loops: bool,
    frametime: f32
}

#[derive(Asset, Deserialize, TypePath)]
pub struct SpriteFrames<> {
    atlas: String,
    #[serde(skip)]
    atlas_handle: Handle<TextureAtlas>,
    states: Map,
}

#[derive(Default)]
pub struct SpriteFramesLoader;

impl AssetLoader for SpriteFramesLoader {
    type Asset = SpriteFrames;
    
    type Settings = bool;

    type Error = SpannedError;

    fn extensions(&self) -> &[&str] {
        &["spriteframes.ron"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf : Vec<u8> = vec![];
            reader.read_to_end(&mut buf);
            let serialized = ron::de::from_bytes::<'_, SpriteFrames>(&buf);
            serialized
        })
    }
}

fn finalize_atlases(
    mut asset_events: EventReader<AssetEvent<SpriteFrames>>,
    mut sprite_frames: ResMut<Assets<SpriteFrames>>
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Added { id } => {
                let frames = sprite_frames.get_mut(*id).unwrap();
            },
            _ => {}
        }
    }
}

/*
pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl Deserialize<'static> for SpriteFrames<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'static> {
        deserializer.deserialize_string(StringVisitor);
        todo!()
    }
}
*/

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

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
    
    
    /*
    let texture_handle = asset_server.load("sprites/atlases/abigail.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(40.0, 96.0), 2, 1, Some(Vec2::new(1., 1.)), Some(Vec2::new(1., 1.)));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 1 };
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)),
    ));
    */
}