use bevy::prelude::*;

use crate::AppState;

pub struct MainMenuPlugin;

fn start_game(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>) {
    if buttons.pressed(MouseButton::Left) {
        commands.set_state(AppState::InGame);
    }
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(AppState::MainMenu)));
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu);
        app.add_systems(Update, start_game.run_if(in_state(AppState::MainMenu)));
    }
}
