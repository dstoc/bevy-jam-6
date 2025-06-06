use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};
use rand::prelude::*;

use crate::{GameRunState, GameState, materials::link_material::LinkMaterial};

use super::{scaling::Scaling, ship::Ship};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct StarfieldMaterial {
    #[uniform(0)]
    pub camera: Vec2,
}

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
    lumina_material: Handle<ColorMaterial>,
    lumina_cooldown_material: Handle<ColorMaterial>,
    line_mesh: Handle<Mesh>,
    link_material: Handle<LinkMaterial>,
}

#[derive(Resource, Default)]
struct Chunks {
    created: HashMap<IVec2, Entity>,
}

#[derive(Component)]
pub struct Attached {
    pub lumina: Entity,
    pub in_range: bool,
}

#[derive(Component, Default)]
pub struct Lumina {
    pub targets: HashSet<Entity>,
}

#[derive(Component)]
pub struct Cooldown;

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

#[derive(Component)]
#[relationship(relationship_target = Contains)]
struct ContainedBy(Entity);

#[derive(Component)]
#[relationship_target(relationship = ContainedBy)]
struct Contains(Vec<Entity>);

#[derive(Component, Default)]
struct Nearby;

#[derive(Event)]
struct AttachedChangeEvent {
    from: Entity,
    to: Entity,
}

#[derive(Component)]
struct AttachmentLine;

const CHUNK_SIZE: f32 = 5000.0;
const CELLS_PER_CHUNK: i32 = 10;
const RESOURCE_DECAY_RATE: f32 = 0.2;
const NEARBY_DISTANCE: f32 = 300.0;
const ATTACH_DISTANCE: f32 = 100.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut starfield_materials: ResMut<Assets<StarfieldMaterial>>,
    mut link_materials: ResMut<Assets<LinkMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(ChunkResources {
        material: starfield_materials.add(StarfieldMaterial {
            camera: Vec2::default(),
        }),
        mesh: meshes.add(Rectangle {
            half_size: Vec2 {
                x: CHUNK_SIZE / 2.0,
                y: CHUNK_SIZE / 2.0,
            },
        }),
        lumina_material: color_materials.add(ColorMaterial::from(Color::srgb(0.8, 0.3, 0.3))),
        lumina_cooldown_material: color_materials
            .add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3))),
        resource_mesh: meshes.add(Circle::new(20.0)).into(),
        line_mesh: meshes.add(Mesh::from(Rectangle::default())),
        link_material: link_materials.add(LinkMaterial {
            base_color: LinearRgba::rgb(0.5, 0.5, 0.0),
            noise_freq: 0.02,
            noise_speed: 2.0,
        }),
    });
}

fn update_starfield(
    resources: Res<ChunkResources>,
    camera_transform: Single<&Transform, With<Ship>>,
    mut starfield_materials: ResMut<Assets<StarfieldMaterial>>,
) {
    let material = starfield_materials.get_mut(&resources.material).unwrap();
    material.camera = camera_transform.translation.xy();
}

fn update_attachment_line(
    ship_transform: Single<&Transform, With<Ship>>,
    lumina: Query<(&Transform, &Lumina)>,
    attached: Option<Single<&Attached>>,
    scaling: Res<Scaling>,
    mut line: Single<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial2d<LinkMaterial>,
        ),
        (With<AttachmentLine>, Without<Lumina>, Without<Ship>),
    >,
    mut link_materials: ResMut<Assets<LinkMaterial>>,
) {
    let end = ship_transform.translation.xy();
    let mut visibility = Visibility::Hidden;
    if let Some(ref attached) = attached {
        if let Ok((lumina_transform, lumina)) = lumina.get(attached.lumina) {
            if attached.in_range || lumina.targets.len() < scaling.max_links {
                let start = lumina_transform.translation.xy();
                *line.0 = transform_for_line(start, end, 20.0);
                visibility = Visibility::Visible;
                let link_material = link_materials.get_mut(&line.2.0).unwrap();
                link_material.base_color = if attached.in_range {
                    LinearRgba::rgb(0.5, 0.5, 0.0)
                } else {
                    LinearRgba::rgb(0.1, 0.1, 0.1)
                };
            }
        }
    }
    if *line.1 != visibility {
        *line.1 = visibility;
    }
}

fn update_nearby_lumina(
    mut commands: Commands,
    chunk_map: Res<Chunks>,
    ship: Single<(Entity, &Transform, Option<&Attached>), With<Ship>>,
    chunks: Query<&Contains, With<Chunk>>,
    lumina: Query<(Entity, &Transform, Option<&Nearby>), With<Lumina>>,
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
                        let distance = lumina_transform.translation.distance(ship.1.translation);
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
        if if let Some(attached) = ship.2 {
            if attached.lumina != lumina {
                attached_events.write(AttachedChangeEvent {
                    from: attached.lumina,
                    to: lumina,
                });
                true
            } else {
                !attached.in_range
            }
        } else {
            true
        } {
            commands.entity(ship.0).insert(Attached {
                lumina,
                in_range: true,
            });
        }
    } else if let Some(attached) = ship.2 {
        if attached.in_range {
            commands.entity(ship.0).insert(Attached {
                in_range: false,
                ..*attached
            });
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
                let chunk_index = chunk + IVec2 { x: dx, y: dy };
                if chunks.created.contains_key(&chunk_index) {
                    continue;
                }
                let chunk_position = Vec2 {
                    x: chunk_index.x as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                    y: chunk_index.y as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
                };
                let chunk_entity = commands
                    .spawn((
                        Chunk,
                        Name::from("Chunk"),
                        StateScoped(GameState::Playing),
                        Mesh2d(resources.mesh.clone()),
                        MeshMaterial2d(resources.material.clone()),
                        Transform::from_xyz(chunk_position.x, chunk_position.y, -10.0),
                    ))
                    .id();
                for xi in 0..CELLS_PER_CHUNK {
                    for yi in 0..CELLS_PER_CHUNK {
                        let distance = (chunk_index * CELLS_PER_CHUNK
                            + IVec2 {
                                x: xi as i32,
                                y: yi as i32,
                            })
                        .as_vec2()
                        .length();
                        let probability = (-RESOURCE_DECAY_RATE * distance).exp() * 0.8 + 0.01;
                        if rand::rng().random_range(0.0..1.0) > probability {
                            continue;
                        }
                        let x_offset =
                            rand::rng().random_range(-0.4..0.4) * cell_size - CHUNK_SIZE / 2.0;
                        let y_offset =
                            rand::rng().random_range(-0.4..0.4) * cell_size - CHUNK_SIZE / 2.0;
                        let position = Vec2 {
                            x: (0.5 + xi as f32) * cell_size + x_offset,
                            y: (0.5 + yi as f32) * cell_size + y_offset,
                        };
                        commands.spawn((
                            Lumina::default(),
                            Name::from("Lumina"),
                            ContainedBy(chunk_entity),
                            StateScoped(GameState::Playing),
                            Mesh2d(resources.resource_mesh.clone()),
                            MeshMaterial2d(resources.lumina_material.clone()),
                            Transform::from_xyz(
                                position.x + chunk_position.x,
                                position.y + chunk_position.y,
                                1.0,
                            ),
                        ));
                    }
                }
                chunks.created.insert(chunk_index, chunk_entity);
            }
        }
    }
}

fn create_links(
    mut commands: Commands,
    mut attached: EventReader<AttachedChangeEvent>,
    mut lumina: Query<(Entity, &Transform, &mut Lumina)>,
    mut disjoint_set: ResMut<LuminaDisjointSet>,
    scaling: Res<Scaling>,
    resources: Res<ChunkResources>,
) {
    for AttachedChangeEvent { from, to } in attached.read() {
        if let Ok(
            [
                (from_entity, from_transform, mut from_lumina),
                (to_entity, to_transform, mut to_lumina),
            ],
        ) = lumina.get_many_mut([*from, *to])
        {
            if !disjoint_set.set.is_linked(from_entity, to_entity)
                && from_lumina.targets.len() < scaling.max_links
                && to_lumina.targets.len() < scaling.max_links
            {
                disjoint_set.set.link(from_entity, to_entity);
                from_lumina.targets.insert(to_entity);
                to_lumina.targets.insert(from_entity);
                commands.spawn((
                    StateScoped(GameState::Playing),
                    Mesh2d(resources.line_mesh.clone()),
                    MeshMaterial2d(resources.link_material.clone()),
                    transform_for_line(
                        from_transform.translation.xy(),
                        to_transform.translation.xy(),
                        20.0,
                    ),
                ));
            }
        }
    }
}

fn transform_for_line(p0: Vec2, p1: Vec2, thickness: f32) -> Transform {
    let delta = p1 - p0;
    let length = delta.length();
    if length < f32::EPSILON {
        return Transform::default();
    }
    let direction = delta / length;
    let midpoint = (p0 + p1) * 0.5;
    let dir3 = Vec3::new(direction.x, direction.y, 0.0);
    let rotation = Quat::from_rotation_arc(Vec3::X, dir3);

    Transform {
        translation: Vec3::new(midpoint.x, midpoint.y, -5.0),
        rotation,
        scale: Vec3::new(length, thickness, 1.0),
    }
}

fn lumina_cooldown_started(
    mut query: Query<&mut MeshMaterial2d<ColorMaterial>, Added<Cooldown>>,
    resources: Res<ChunkResources>,
) {
    for mut mesh_material in query.iter_mut() {
        mesh_material.0 = resources.lumina_cooldown_material.clone();
    }
}
fn lumina_cooldown_ended(
    mut removed: RemovedComponents<Cooldown>,
    mut query: Query<&mut MeshMaterial2d<ColorMaterial>>,
    resources: Res<ChunkResources>,
) {
    for entity in removed.read() {
        if let Ok(mut mesh_material) = query.get_mut(entity) {
            mesh_material.0 = resources.lumina_material.clone();
        }
    }
}

fn setup_game(
    mut commands: Commands,
    resources: Res<ChunkResources>,
    mut link_materials: ResMut<Assets<LinkMaterial>>,
) {
    commands.insert_resource(Chunks::default());
    commands.insert_resource(LuminaDisjointSet::default());
    commands.spawn((
        AttachmentLine,
        StateScoped(GameState::Playing),
        Mesh2d(resources.line_mesh.clone()),
        MeshMaterial2d(link_materials.add(LinkMaterial {
            base_color: LinearRgba::rgb(0.0, 1.0, 1.0),
            noise_freq: 0.02,
            noise_speed: 2.0,
        })),
        Transform::default(),
        Visibility::Hidden,
    ));
}

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<LinkMaterial>::default())
            .add_plugins(Material2dPlugin::<StarfieldMaterial>::default())
            .add_systems(
                Update,
                (
                    update_nearby_lumina,
                    update_attachment_line,
                    update_starfield,
                )
                    .run_if(in_state(GameRunState::Playing).or(in_state(GameRunState::Ending))),
            )
            .add_systems(
                Update,
                (
                    populate_nearby_chunks,
                    create_links,
                    lumina_cooldown_started,
                    lumina_cooldown_ended,
                    create_links,
                )
                    .run_if(in_state(GameRunState::Playing)),
            )
            .add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(Startup, setup)
            .add_event::<AttachedChangeEvent>();
    }
}
