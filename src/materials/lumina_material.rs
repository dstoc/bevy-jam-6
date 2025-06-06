use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LuminaMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,

    #[uniform(0)]
    pub fill_color: LinearRgba,

    #[uniform(0)]
    pub bloom: f32,

    #[uniform(0)]
    pub freq: f32,
}

impl Material2d for LuminaMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lumina.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
