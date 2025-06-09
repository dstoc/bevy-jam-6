use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::GameRunState;

use super::ship::Ship;

pub struct PauseMenuPlugin;

fn abandon_ship(
    _trigger: Trigger<Pointer<Click>>,
    mut ship: Single<&mut Ship>,
    mut commands: Commands,
) {
    ship.energy = 0.0;
    commands.set_state(GameRunState::Playing);
}

fn resume_playing(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(GameRunState::Playing);
}

fn pause(mut commands: Commands) {
    commands.set_state(GameRunState::Paused);
}

#[cfg(not(target_arch = "wasm32"))]
fn quit_game(_trigger: Trigger<Pointer<Click>>, mut exit_events: EventWriter<AppExit>) {
    exit_events.write(AppExit::default());
}

fn button(text: &str) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(175.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        BackgroundColor(Color::BLACK),
        BorderRadius::all(Val::Px(5.0)),
        children![(Text::new(text), TextColor(Color::WHITE))],
    )
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            StateScoped(GameRunState::Paused),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(button("Abandon ship")).observe(abandon_ship);
            #[cfg(not(target_arch = "wasm32"))]
            parent.spawn(button("Quit game")).observe(quit_game);
            parent.spawn(button("Continue run")).observe(resume_playing);
        });
}

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameRunState::Paused), setup)
            .add_systems(
                Update,
                pause.run_if(
                    in_state(GameRunState::Playing).and(input_just_pressed(KeyCode::Escape)),
                ),
            );
    }
}
