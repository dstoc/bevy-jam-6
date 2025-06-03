use bevy::{
    platform::collections::{HashMap, HashSet},
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
    created: HashMap<IVec2, Entity>,
}

#[derive(Component)]
struct Attached {
    lumina: Entity,
}

#[derive(Component, Default)]
#[require(Links)]
struct Lumina;

#[derive(Component, Default)]
struct Links {
    targets: HashSet<Entity>,
}

#[derive(Resource)]
struct LuminaDisjointSet {
    set: disjoint_hash_set::DisjointHashSet<Entity>,
}

impl Default for LuminaDisjointSet {
    fn default() -> Self {
        Self {
            set: disjoint_hash_set::DisjointHashSet::new(),
        }
    }
}

#[derive(Component, Default)]
struct Chunk;

#[derive(Component, Default)]
struct Nearby;

#[derive(Event)]
struct AttachedChangeEvent {
    from: Entity,
    to: Entity,
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

const NEARBY_DISTANCE: f32 = 300.0;
const ATTACH_DISTANCE: f32 = 100.0;

fn test_draw_lines(
    mut gizmos: Gizmos,
    ship_transform: Single<&Transform, With<Ship>>,
    nearby: Query<(Entity, &GlobalTransform), With<Nearby>>,
    lumina: Query<&GlobalTransform, With<Lumina>>,
    attached: Option<Single<&Attached>>,
    links: Query<(Entity, &Links)>,
) {
    let start_point = ship_transform.translation.xy();
    for (nearby, nearby_transform) in nearby.iter() {
        if attached
            .as_ref()
            .map_or(false, |attached| nearby == attached.lumina)
        {
            continue;
        }
        let end_point = nearby_transform.translation().xy();
        gizmos.line_2d(start_point, end_point, Color::srgb(1.0, 0.0, 0.0));
    }
    if let Some(attached) = attached {
        if let Ok(lumina_transform) = lumina.get(attached.lumina) {
            let end_point = lumina_transform.translation().xy();
            gizmos.line_2d(start_point, end_point, Color::srgb(0.0, 0.0, 1.0));
        }
    }
    for (source, links) in links.iter() {
        for target in links.targets.iter() {
            if source < *target {
                let start_point = lumina.get(source).unwrap().translation().xy();
                let end_point = lumina.get(*target).unwrap().translation().xy();
                gizmos.line_2d(start_point, end_point, Color::srgb(0.0, 1.0, 0.0));
            }
        }
    }
}

fn update_nearby_lumina(
    mut commands: Commands,
    chunk_map: Res<Chunks>,
    ship: Single<(Entity, &Transform, Option<&Attached>), With<Ship>>,
    chunks: Query<&Children, With<Chunk>>,
    lumina: Query<(Entity, &GlobalTransform, Option<&Nearby>), With<Lumina>>,
    nearby: Query<Entity, With<Nearby>>,
    mut attached_events: EventWriter<AttachedChangeEvent>,
) {
    let mut validated = HashSet::<Entity>::new();
    let mut closest_distance = f32::INFINITY;
    let mut closest_lumina = Option::<Entity>::None;
    for chunk_position in iter_surrounding_chunks(ship.1.translation.xy()) {
        if let Some(chunk_entity) = chunk_map.created.get(&chunk_position) {
            if let Ok(children) = chunks.get(*chunk_entity) {
                for child in children.iter() {
                    if let Ok((lumina, lumina_transform, nearby)) = lumina.get(child) {
                        let distance = lumina_transform.translation().distance(ship.1.translation);
                        if distance < NEARBY_DISTANCE {
                            validated.insert(lumina);
                            if let None = nearby {
                                commands.entity(lumina).insert(Nearby);
                                validated.insert(lumina);
                            }
                        }
                        if distance < ATTACH_DISTANCE && distance < closest_distance {
                            closest_distance = distance;
                            closest_lumina = Some(lumina);
                        }
                    }
                }
            }
        }
    }
    for nearby in nearby.iter() {
        if !validated.contains(&nearby) {
            commands.entity(nearby).remove::<Nearby>();
        }
    }
    if let Some(lumina) = closest_lumina {
        if let Some(attached) = ship.2 {
            if attached.lumina != lumina {
                attached_events.write(AttachedChangeEvent {
                    from: attached.lumina,
                    to: lumina,
                });
                commands.entity(ship.0).insert(Attached { lumina });
            }
        } else {
            commands.entity(ship.0).insert(Attached { lumina });
        }
    }
}

fn iter_surrounding_chunks(position: Vec2) -> impl Iterator<Item = IVec2> {
    let chunk_base = (position / CHUNK_SIZE).floor().as_ivec2();

    (-1..=1).flat_map(move |dx| (-1..=1).map(move |dy| chunk_base + IVec2 { x: dx, y: dy }))
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
                if chunks.created.contains_key(&chunk) {
                    continue;
                }
                let chunk_position = Vec2 {
                    x: chunk.x as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                    y: chunk.y as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                };
                let chunk_entity = commands
                    .spawn((
                        Chunk,
                        Name::from("Chunk"),
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
                                    Lumina,
                                    Name::from("Lumina"),
                                    Mesh2d(resources.resource_mesh.clone()),
                                    MeshMaterial2d(resources.resource_material.clone()),
                                    Transform::from_xyz(position.x, position.y, 1.0),
                                ));
                            }
                        }
                    })
                    .id();
                chunks.created.insert(chunk, chunk_entity);
            }
        }
    }
}

fn create_links(
    mut attached: EventReader<AttachedChangeEvent>,
    mut links: Query<(Entity, &mut Links)>,
    mut disjoint_set: ResMut<LuminaDisjointSet>,
) {
    for AttachedChangeEvent { from, to } in attached.read() {
        if let Ok([(from_entity, mut from_links), (to_entity, mut to_links)]) =
            links.get_many_mut([*from, *to])
        {
            if !disjoint_set.set.is_linked(from_entity, to_entity) {
                disjoint_set.set.link(from_entity, to_entity);
                from_links.targets.insert(to_entity);
                to_links.targets.insert(from_entity);
            }
        }
    }
}

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<StarfieldMaterial>::default())
            .add_systems(Update, populate_nearby_chunks)
            .add_systems(
                PostUpdate,
                update_nearby_lumina.after(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PostUpdate,
                test_draw_lines.after(TransformSystem::TransformPropagate),
            )
            .add_systems(Update, create_links)
            .add_systems(Startup, setup)
            .add_event::<AttachedChangeEvent>()
            .insert_resource(Chunks::default())
            .insert_resource(LuminaDisjointSet::default());
    }
}
