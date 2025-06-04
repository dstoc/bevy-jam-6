use super::{
    chunks::{Attached, Lumina},
    ship::Ship,
};
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
struct EnergyResources {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
struct Energy {
    target: Entity,
    t: f32,
    path: Vec<Entity>,
    returning: bool,
    distance: f32,
}

fn generate_energy(
    time: Res<Time>,
    mut commands: Commands,
    attached: Option<Single<&Attached>>,
    lumina: Query<&Lumina>,
    resources: Res<EnergyResources>,
) {
    if let Some(ref attached) = attached {
        if !attached.in_range {
            return;
        }
        let lumina = lumina.get(attached.lumina).unwrap();
        for target in lumina.targets.iter() {
            // TODO: move to central storage
            if rand::rng().random_range(0.0..1.0) > time.delta_secs() {
                continue;
            }
            commands.spawn((
                Energy {
                    target: *target,
                    t: 0.0,
                    path: vec![attached.lumina],
                    returning: false,
                    distance: 0.0,
                },
                Mesh2d(resources.mesh.clone()),
                MeshMaterial2d(resources.material.clone()),
                Transform::default(),
            ));
        }
        if lumina.targets.is_empty() {
            // TODO: move to central storage
            if rand::rng().random_range(0.0..1.0) > time.delta_secs() {
                return;
            }
            commands.spawn((
                Energy {
                    target: attached.lumina,
                    t: 1.0,
                    path: vec![],
                    returning: true,
                    distance: 0.0,
                },
                Mesh2d(resources.mesh.clone()),
                MeshMaterial2d(resources.material.clone()),
                Transform::default(),
            ));
        }
    }
}

const SPEED: f32 = 500.0;

fn move_energy(
    mut commands: Commands,
    time: Res<Time>,
    energy: Query<(Entity, &mut Transform, &mut Energy)>,
    lumina: Query<(&GlobalTransform, &Lumina)>,
    resources: Res<EnergyResources>,
) {
    for (entity, mut transform, mut energy) in energy {
        if energy.path.is_empty() {
            continue;
        }
        let from = energy.path.last().unwrap().clone();
        let to = energy.target;
        let from_pos = lumina.get(from).unwrap().0.translation().xy();
        let to_pos = lumina.get(to).unwrap().0.translation().xy();
        let total_distance = from_pos.distance(to_pos);
        let current_distance = energy.t * total_distance;
        let new_distance = current_distance + time.delta().as_secs_f32() * SPEED;
        let new_t = (new_distance / total_distance).clamp(0.0, 1.0);
        let new_pos = from_pos.lerp(to_pos, new_t);

        transform.translation = new_pos.extend(0.0);

        energy.t = new_t;
        if energy.t >= 1.0 {
            energy.distance += total_distance;
            if energy.returning {
                energy.path.pop();
                if energy.path.len() >= 1 {
                    energy.target = energy.path.pop().unwrap();
                    energy.path.push(to);
                    energy.t = 0.0;
                }
            } else {
                let to_lumina = lumina.get(to).unwrap().1;
                if to_lumina.targets.len() == 1 {
                    energy.returning = true;
                    energy.target = energy.path.pop().unwrap();
                    energy.path.push(to);
                    energy.t = 0.0;
                } else {
                    for target in to_lumina.targets.iter() {
                        if *target == from {
                            continue;
                        }
                        let mut path = energy.path.clone();
                        path.push(to);
                        commands.spawn((
                            Energy {
                                target: *target,
                                t: 0.0,
                                path,
                                returning: false,
                                distance: energy.distance,
                            },
                            Mesh2d(resources.mesh.clone()),
                            MeshMaterial2d(resources.material.clone()),
                            Transform::default(),
                        ));
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn deliver_energy(
    mut commands: Commands,
    mut ship: Single<&mut Ship>,
    attached: Option<Single<&Attached>>,
    energy: Query<(Entity, &mut Energy)>,
) {
    for (entity, energy) in energy {
        if attached.as_ref().map_or(false, |attached| {
            attached.in_range && energy.target == attached.lumina
        }) && energy.path.is_empty()
        {
            // energy was emitted at a node with no links
            ship.energy += 500.0; // TODO: ? energy.distance;
            commands.entity(entity).despawn();
        } else if energy.path.is_empty() {
            // energy has finished propagating
            commands.entity(entity).despawn();
        } else if attached.as_ref().map_or(false, |attached| {
            attached.in_range
                && *energy.path.last().unwrap() == attached.lumina
                && (energy.returning || energy.path.len() > 1)
        }) {
            ship.energy += energy.distance;
            commands.entity(entity).despawn();
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(EnergyResources {
        material: color_materials.add(ColorMaterial::from(Color::srgb(0.2, 0.7, 0.8))),
        mesh: meshes.add(Circle::new(10.0)).into(),
    });
}

pub struct EnergyPlugin;

impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, generate_energy)
            .add_systems(Update, deliver_energy)
            .add_systems(Update, move_energy);
    }
}
