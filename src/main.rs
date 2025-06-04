use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};
use bevy_egui::EguiPlugin;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use iyes_perf_ui::{PerfUiPlugin, entries::PerfUiDefaultEntries};
use plugins::{
    chunks::ChunksPlugin,
    energy::EnergyPlugin,
    energy_display::EnergyDisplayPlugin,
    ship::{Ship, ShipPlugin},
};

mod plugins {
    pub mod chunks;
    pub mod energy;
    pub mod energy_display;
    pub mod ship;
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(ChunksPlugin)
    .add_plugins(ShipPlugin)
    .add_plugins(EnergyPlugin)
    .add_plugins(EnergyDisplayPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, zoom_camera);
    #[cfg(debug_assertions)]
    {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin);
    }
    app.run();
}

fn zoom_camera(
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut Projection, With<Camera>>,
) {
    if mouse_scroll.delta == Vec2::ZERO {
        return;
    }

    for mut projection in query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale -= mouse_scroll.delta.y * 0.1;
            ortho.scale = ortho.scale.clamp(0.1, 10.0);
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    #[cfg(debug_assertions)]
    {
        commands.spawn(PerfUiDefaultEntries::default());
    }
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(20.0)).into()),
        MeshMaterial2d(color_materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8)))),
        Transform::default(),
        Ship::default(),
        Name::from("Ship"),
        Camera2d,
    ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(20.0)).into()),
        MeshMaterial2d(color_materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8)))),
        Transform::default(),
    ));
}
