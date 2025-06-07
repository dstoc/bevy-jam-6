use bevy::{prelude::*, window::WindowResized};
use bevy_egui::EguiPlugin;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::TweeningPlugin;
#[cfg(debug_assertions)]
use iyes_perf_ui::{PerfUiPlugin, entries::PerfUiDefaultEntries};
use plugins::{
    chunks::ChunksPlugin, energy::EnergyPlugin, energy_display::EnergyDisplayPlugin,
    game_loop::GameLoopPlugin, main_menu::MainMenuPlugin, pause_menu::PauseMenuPlugin,
    scaling::ScalingPlugin, ship::ShipPlugin, shop::ShopPlugin, story::StoryPlugin,
};

mod plugins {
    pub mod chunks;
    pub mod energy;
    pub mod energy_display;
    pub mod game_loop;
    pub mod main_menu;
    pub mod pause_menu;
    pub mod scaling;
    pub mod ship;
    pub mod shop;
    pub mod story;
}

mod materials {
    pub mod link_material;
    pub mod lumina_material;
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
    Story,
    Shop,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Playing)]
#[states(scoped_entities)]
pub enum GameRunState {
    #[default]
    // Starting,
    Playing,
    Paused,
    Ending,
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
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(UiScale::default())
    .add_sub_state::<GameState>()
    .add_sub_state::<GameRunState>()
    .add_plugins(MainMenuPlugin)
    .add_plugins(PauseMenuPlugin)
    .add_plugins(GameLoopPlugin)
    .add_plugins(StoryPlugin)
    .add_plugins(ShopPlugin)
    .add_plugins(ChunksPlugin)
    .add_plugins(ShipPlugin)
    .add_plugins(EnergyPlugin)
    .add_plugins(EnergyDisplayPlugin)
    .add_plugins(ScalingPlugin)
    .add_plugins(TweeningPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, init_camera)
    .add_systems(Update, resize_camera);
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

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;
const SCALE: f32 = 1.5;

fn init_camera(projection: Query<&mut Projection, Added<Camera2d>>, window: Single<&Window>) {
    let scale_x = window.width() / WIDTH;
    let scale_y = window.height() / HEIGHT;
    let zoom = (scale_x.min(scale_y)).recip();
    for mut projection in projection {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = zoom * SCALE;
        }
    }
}

fn resize_camera(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Projection, With<Camera2d>>,
    mut ui_scale: ResMut<UiScale>,
) {
    for e in resize_events.read() {
        let scale_x = e.width / WIDTH;
        let scale_y = e.height / HEIGHT;
        let zoom = (scale_x.min(scale_y)).recip();
        ui_scale.0 = 1.0 / zoom;
        for mut projection in query.iter_mut() {
            if let Projection::Orthographic(ref mut ortho) = *projection {
                ortho.scale = zoom * SCALE;
            }
        }
    }
}

fn setup(mut commands: Commands) {
    #[cfg(debug_assertions)]
    {
        commands.spawn(PerfUiDefaultEntries::default());
    }
}
