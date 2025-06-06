use bevy::prelude::*;

use crate::GameState;

pub struct StoryPlugin;

fn go_next(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(GameState::Shop);
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(GameState::Story)));
    commands
        .spawn((
            StateScoped(GameState::Story),
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
                        Text::new("Story?"),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )],
                ))
                .observe(go_next);
        });
}

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Story), setup);
    }
}
