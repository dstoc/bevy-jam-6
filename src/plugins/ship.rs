use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::GameRunState;

use super::scaling::Scaling;

#[derive(Component)]
pub struct ShipSprite;

#[derive(Component)]
pub struct Ship {
    pub linear: Vec2,
    pub energy: f32,
}

fn ship_movement(
    ship: Single<(&Transform, &mut Ship, &Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    time: Res<Time>,
    scaling: Res<Scaling>,
    mut ship_sprite: Single<&mut Transform, (With<ShipSprite>, Without<Ship>)>,
) {
    let (ship_transform, mut ship, camera, camera_transform) = ship.into_inner();
    let force_magnitude = 500.0;
    let dt = time.delta_secs();

    ship.energy = 0.0f32.max(
        ship.energy
            - ship_transform.translation.distance(Vec3::ZERO)
                * scaling.life_support_per_distance
                * dt,
    );

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        if buttons.pressed(MouseButton::Left) {
            let direction = (world_pos - ship_transform.translation.xy()).normalize_or_zero();
            let force = ship.energy.min(force_magnitude * dt);
            ship.linear += direction * force;
            ship.energy -= force * scaling.energy_per_force;
            ship_sprite.rotation = Quat::from_rotation_z(direction.to_angle() - FRAC_PI_2);
        } else if buttons.pressed(MouseButton::Right) {
            if ship.linear.length_squared() > f32::EPSILON {
                let force = ship.energy.min(force_magnitude * dt);
                let braking_force_vector = -ship.linear.normalize() * force;
                ship.energy -= force * scaling.energy_per_force;
                ship_sprite.rotation =
                    Quat::from_rotation_z(braking_force_vector.to_angle() - FRAC_PI_2);
                if ship.linear.dot(ship.linear + braking_force_vector) < 0.0 {
                    ship.linear = Vec2::ZERO;
                } else {
                    ship.linear += braking_force_vector;
                }
            } else {
                ship.linear = Vec2::ZERO;
            }
        }
    }
    if ship.energy > scaling.max_battery {
        ship.energy -=
            (scaling.capacitor_drain_per_sec * dt).min(ship.energy - scaling.max_battery);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Ship)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.linear.extend(0.0) * dt;
    }
}

fn playing_or_ending(state: Option<Res<State<GameRunState>>>) -> bool {
    state.map_or(false, |state| {
        *state == GameRunState::Playing || *state == GameRunState::Ending
    })
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ship_movement.run_if(in_state(GameRunState::Playing)),
        )
        .add_systems(Update, apply_velocity.run_if(playing_or_ending));
    }
}
