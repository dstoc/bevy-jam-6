use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};
use bevy_egui::EguiPlugin;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
#[cfg(debug_assertions)]
use iyes_perf_ui::{PerfUiPlugin, entries::PerfUiDefaultEntries};
use plugins::{
    chunks::ChunksPlugin, energy::EnergyPlugin, energy_display::EnergyDisplayPlugin,
    game_loop::GameLoopPlugin, main_menu::MainMenuPlugin, scaling::ScalingPlugin, ship::ShipPlugin,
};

mod plugins {
    pub mod chunks;
    pub mod energy;
    pub mod energy_display;
    pub mod game_loop;
    pub mod main_menu;
    pub mod scaling;
    pub mod ship;
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    Playing,
    // Shop,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
#[states(scoped_entities)]
pub enum GameRunState {
    #[default]
    Playing,
    // Paused,
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
    .init_state::<AppState>()
    .add_sub_state::<GameState>()
    .add_sub_state::<GameRunState>()
    .add_plugins(MainMenuPlugin)
    .add_plugins(GameLoopPlugin)
    .add_plugins(ChunksPlugin)
    .add_plugins(ShipPlugin)
    .add_plugins(EnergyPlugin)
    .add_plugins(EnergyDisplayPlugin)
    .add_plugins(ScalingPlugin)
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

fn setup(mut commands: Commands) {
    #[cfg(debug_assertions)]
    {
        commands.spawn(PerfUiDefaultEntries::default());
    }
}
