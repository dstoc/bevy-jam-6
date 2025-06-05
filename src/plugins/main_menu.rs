use bevy::prelude::*;

use crate::AppState;

pub struct MainMenuPlugin;

fn start_game(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(AppState::InGame);
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(AppState::MainMenu)));
    commands
        .spawn((
            StateScoped(AppState::MainMenu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BorderRadius::MAX,
                    children![(
                        Text::new("Start"),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )],
                ))
                .observe(start_game);
        });
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_menu);
    }
}
