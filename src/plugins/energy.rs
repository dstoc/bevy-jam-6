use crate::{GameRunState, GameState};

use super::{
    chunks::{Attached, Cooldown, Lumina},
    scaling::Scaling,
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

fn resume_lumina(
    mut commands: Commands,
    attached: Option<Single<&Attached>>,
    cooldown: Query<Entity, With<Cooldown>>,
    time: Res<Time>,
    scaling: Res<Scaling>,
) {
    for entity in cooldown.iter() {
        if attached
            .as_ref()
            .map_or(false, |attached| attached.lumina == entity)
        {
            // don't end cooldown while attached
            continue;
        }
        if rand::rng().random_range(0.0..1.0) < scaling.lumina_resume_per_sec * time.delta_secs() {
            commands.entity(entity).remove::<Cooldown>();
        }
    }
}

fn generate_energy(
    time: Res<Time>,
    mut commands: Commands,
    attached: Option<Single<&Attached>>,
    lumina: Query<(&Transform, &Lumina)>,
    cooldown: Query<&Cooldown>,
    resources: Res<EnergyResources>,
    scaling: Res<Scaling>,
) {
    if let Some(ref attached) = attached {
        if !attached.in_range {
            return;
        }
        let (transform, lumina) = lumina.get(attached.lumina).unwrap();
        if cooldown.contains(attached.lumina) {
            return;
        }
        for target in lumina.targets.iter() {
            if rand::rng().random_range(0.0..1.0) > scaling.generation_per_sec * time.delta_secs() {
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
                Name::from("Energy"),
                StateScoped(GameState::Playing),
                Mesh2d(resources.mesh.clone()),
                MeshMaterial2d(resources.material.clone()),
                transform.clone(),
            ));
            if rand::rng().random_range(0.0..1.0) < scaling.lumina_cooldown_per_generation {
                commands.entity(attached.lumina).insert(Cooldown);
                return;
            }
        }
    }
}

const SPEED: f32 = 500.0;

fn move_energy(
    mut commands: Commands,
    time: Res<Time>,
    energy: Query<(Entity, &mut Transform, &mut Energy), Without<Lumina>>,
    lumina: Query<(&Transform, &Lumina)>,
    resources: Res<EnergyResources>,
    scaling: Res<Scaling>,
) {
    for (entity, mut transform, mut energy) in energy {
        if energy.path.is_empty() {
            continue;
        }
        let from = energy.path.last().unwrap().clone();
        let to = energy.target;
        let from_pos = lumina.get(from).unwrap().0.translation.xy();
        let to_pos = lumina.get(to).unwrap().0.translation.xy();
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
                // Continue returning
                energy.path.pop();
                if energy.path.len() >= 1 {
                    energy.target = energy.path.pop().unwrap();
                    energy.path.push(to);
                    if rand::rng().random_range(0.0..1.0) > scaling.propagation_probability {
                        energy.path.clear();
                    }
                    energy.t = 0.0;
                }
            } else {
                let to_lumina = lumina.get(to).unwrap().1;
                for target in to_lumina.targets.iter() {
                    let terminated = if *target == from {
                        rand::rng().random_range(0.0..1.0) > scaling.reflection_probability
                    } else {
                        rand::rng().random_range(0.0..1.0) > scaling.propagation_probability
                    };
                    let path = if terminated {
                        vec![]
                    } else {
                        let mut path = energy.path.clone();
                        path.push(to);
                        path
                    };
                    commands.spawn((
                        Energy {
                            target: *target,
                            t: 0.0,
                            path,
                            returning: *target == from,
                            distance: energy.distance,
                        },
                        Name::from("Energy"),
                        StateScoped(GameState::Playing),
                        Mesh2d(resources.mesh.clone()),
                        MeshMaterial2d(resources.material.clone()),
                        Transform::from_translation(new_pos.extend(0.0)),
                    ));
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

fn deliver_energy(
    mut commands: Commands,
    mut ship: Single<&mut Ship>,
    attached: Option<Single<&Attached>>,
    energy: Query<(Entity, &mut Energy)>,
    scaling: Res<Scaling>,
) {
    for (entity, energy) in energy {
        if attached.as_ref().map_or(false, |attached| {
            attached.in_range && energy.target == attached.lumina
        }) && energy.path.is_empty()
        {
            ship.energy += energy.distance * scaling.energy_extraction;
            commands.entity(entity).despawn();
        } else if energy.path.is_empty() {
            // energy has finished propagating
            commands.entity(entity).despawn();
        } else if attached.as_ref().map_or(false, |attached| {
            attached.in_range
                && *energy.path.last().unwrap() == attached.lumina
                && (energy.returning || energy.path.len() > 1)
        }) {
            ship.energy += energy.distance * scaling.energy_extraction;
            commands.entity(entity).despawn();
        }
    }
    ship.energy = ship.energy.min(scaling.max_battery + scaling.max_capacitor);
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
            .add_systems(
                Update,
                (generate_energy, deliver_energy, resume_lumina)
                    .run_if(in_state(GameRunState::Playing)),
            )
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (move_energy)
                    .run_if(in_state(GameRunState::Playing).or(in_state(GameRunState::Ending))),
            );
    }
}
