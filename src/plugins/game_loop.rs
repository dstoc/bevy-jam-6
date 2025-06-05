use bevy::prelude::*;

use crate::{AppState, GameRunState, GameState};

use super::ship::Ship;

pub struct GameLoopPlugin;

fn setup_run(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::from("Ship"),
        Ship::default(),
        StateScoped(GameState::Playing),
        Mesh2d(meshes.add(Circle::new(20.0)).into()),
        MeshMaterial2d(color_materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8)))),
        Transform::default(),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn check_run(mut commands: Commands, ship: Single<&Ship>) {
    if ship.energy <= 0.0 {
        commands.set_state(AppState::MainMenu);
    }
}

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_run)
            .add_systems(Update, check_run.run_if(in_state(GameRunState::Playing)));
    }
}
