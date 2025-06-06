use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::GameRunState;

pub struct PauseMenuPlugin;

fn go_next(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(GameRunState::Playing);
}

fn pause(mut commands: Commands) {
    commands.set_state(GameRunState::Paused);
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            StateScoped(GameRunState::Paused),
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
                        Text::new("Pause?"),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )],
                ))
                .observe(go_next);
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
