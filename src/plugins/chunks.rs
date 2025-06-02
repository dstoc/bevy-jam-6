use bevy::{
    platform::collections::HashSet,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};
use rand::prelude::*;

use super::ship::Ship;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct StarfieldMaterial {}

impl Material2d for StarfieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }
}

#[derive(Resource, Default)]
struct ChunkResources {
    material: Handle<StarfieldMaterial>,
    mesh: Handle<Mesh>,
    resource_mesh: Handle<Mesh>,
    resource_material: Handle<ColorMaterial>,
}

#[derive(Resource, Default)]
struct Chunks {
    created: HashSet<IVec2>,
}

const CHUNK_SIZE: f32 = 5000.0;
const CELLS_PER_CHUNK: i32 = 10;
const RESOURCE_DECAY_RATE: f32 = 0.2;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StarfieldMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(ChunkResources {
        material: materials.add(StarfieldMaterial {}),
        mesh: meshes.add(Rectangle {
            half_size: Vec2 {
                x: CHUNK_SIZE / 2.0,
                y: CHUNK_SIZE / 2.0,
            },
        }),
        resource_material: color_materials.add(ColorMaterial::from(Color::srgb(0.8, 0.3, 0.3))),
        resource_mesh: meshes.add(Circle::new(20.0)).into(),
    });
}

fn populate_nearby_chunks(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    resources: Res<ChunkResources>,
    transform: Query<&Transform, With<Ship>>,
) {
    if let Ok(transform) = transform.single() {
        let position = transform.translation.xy();
        let chunk = (position / CHUNK_SIZE).floor().as_ivec2();
        let cell_size = CHUNK_SIZE / CELLS_PER_CHUNK as f32;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let chunk = chunk + IVec2 { x: dx, y: dy };
                if chunks.created.contains(&chunk) {
                    continue;
                }
                let chunk_position = Vec2 {
                    x: chunk.x as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                    y: chunk.y as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                };
                chunks.created.insert(chunk);
                commands
                    .spawn((
                        Mesh2d(resources.mesh.clone()),
                        MeshMaterial2d(resources.material.clone()),
                        Transform::from_xyz(chunk_position.x, chunk_position.y, -1.0),
                    ))
                    .with_children(|parent| {
                        for xi in 0..CELLS_PER_CHUNK {
                            for yi in 0..CELLS_PER_CHUNK {
                                let distance = (chunk * CELLS_PER_CHUNK
                                    + IVec2 {
                                        x: xi as i32,
                                        y: yi as i32,
                                    })
                                .as_vec2()
                                .length();
                                let probability =
                                    (-RESOURCE_DECAY_RATE * distance).exp() * 0.8 + 0.01;
                                if rand::rng().random_range(0.0..1.0) > probability {
                                    continue;
                                }
                                let x_offset = rand::rng().random_range(-0.4..0.4) * cell_size
                                    - CHUNK_SIZE / 2.0;
                                let y_offset = rand::rng().random_range(-0.4..0.4) * cell_size
                                    - CHUNK_SIZE / 2.0;
                                let position = Vec2 {
                                    x: (0.5 + xi as f32) * cell_size + x_offset,
                                    y: (0.5 + yi as f32) * cell_size + y_offset,
                                };
                                parent.spawn((
                                    Mesh2d(resources.resource_mesh.clone()),
                                    MeshMaterial2d(resources.resource_material.clone()),
                                    Transform::from_xyz(position.x, position.y, 1.0),
                                ));
                            }
                        }
                    });
            }
        }
    }
}

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<StarfieldMaterial>::default())
            .add_systems(Update, populate_nearby_chunks)
            .add_systems(Startup, setup)
            .insert_resource(Chunks::default());
    }
}
