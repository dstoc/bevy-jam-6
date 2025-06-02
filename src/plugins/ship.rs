use bevy::prelude::*;

#[derive(Component)]
#[require(Velocity)]
pub struct Ship;

#[derive(Component, Default)]
struct Velocity {
    linear: Vec2,
}

fn ship_movement(
    mut query: Query<(&Transform, &mut Velocity, &Camera, &GlobalTransform), With<Ship>>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    time: Res<Time>,
) {
    let (ship_transform, mut ship_velocity, camera, camera_transform) = query.single_mut().unwrap();
    let force_magnitude = 500.0;
    let dt = time.delta_secs();

    if let Some(world_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        if buttons.pressed(MouseButton::Left) {
            let direction = (world_pos - ship_transform.translation.xy()).normalize_or_zero();
            ship_velocity.linear += direction * force_magnitude * dt;
        } else if buttons.pressed(MouseButton::Right) {
            if ship_velocity.linear.length_squared() > f32::EPSILON {
                let braking_force_vector = -ship_velocity.linear.normalize() * force_magnitude * dt;
                if ship_velocity
                    .linear
                    .dot(ship_velocity.linear + braking_force_vector)
                    < 0.0
                {
                    ship_velocity.linear = Vec2::ZERO;
                } else {
                    ship_velocity.linear += braking_force_vector;
                }
            } else {
                ship_velocity.linear = Vec2::ZERO;
            }
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity), With<Ship>>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.linear.extend(0.0) * dt;
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ship_movement)
            .add_systems(FixedUpdate, apply_velocity);
    }
}
