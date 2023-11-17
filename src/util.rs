use std::time::Duration;

use bevy::{
    math::Rect,
    sprite::TextureAtlasSprite, ecs::component::Component, prelude::{Deref, DerefMut}, time::Timer,
};

#[derive(Component, Clone, Copy)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

impl PartialEq for AnimationIndices {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.last == other.last
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);


pub struct Animator {
    pub index: usize,
    pub timer: AnimationTimer,
    pub indices: AnimationIndices,
    pub loops: bool,
}

impl Animator {
    pub fn new(
        timer: AnimationTimer,
        indices: AnimationIndices,
        loops: bool,
    ) -> Animator {
        Animator {
            index: 0,
            timer: timer,
            indices: indices,
            loops: loops,
        }
    }

    pub fn tick(self: &mut Self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.just_finished() {
            if self.index == self.indices.last {
                if self.loops {
                    println!("Loop");
                    self.index = self.indices.first
                } else {
                    println!("End");
                }
            } else {
                println!("Step");
                self.index += 1;
            };
            println!("Sprite Index: {}, Loops: {}, First: {}", self.index, self.loops, self.indices.first);
        }
    }

    pub fn set_frametime(self: &mut Self, secs: f32) {
        self.timer.set_duration(Duration::from_secs_f32(secs));
    }

    pub fn set_indices(self: &mut Self, indices: AnimationIndices) {
        if indices != self.indices {
            self.indices = indices;
            self.index = indices.first;
            self.timer.reset();
        }
    }

    pub fn set_indices_from_bounds(self: &mut Self, first: usize, last: usize) {
        self.indices = AnimationIndices {
            first: first,
            last: last,
        };
    }
}

pub struct AtlasUtil;
impl AtlasUtil {
    pub fn from_corner_size(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(x, y, x + w, y + h)
    }
}
