use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LinkMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,

    #[uniform(0)]
    pub bloom: f32,

    #[uniform(0)]
    pub noise_freq: f32,

    #[uniform(0)]
    pub noise_speed: f32,
}

impl Material2d for LinkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/link.wgsl".into()
    }
    // TODO: blend + bloom seems broken in webgl
    #[cfg(not(target_arch = "wasm32"))]
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
