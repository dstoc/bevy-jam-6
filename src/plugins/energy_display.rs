use bevy::{color::palettes::css, prelude::*};

use crate::{GameRunState, GameState};

use super::{scaling::Scaling, ship::Ship};

#[derive(Component)]
struct EnergyText;

#[derive(Component)]
struct EnergyFill;

fn setup_game(mut commands: Commands) {
    commands.spawn((
        StateScoped(GameState::Playing),
        Text::new("Energy: 0.0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextLayout { ..default() },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        EnergyText,
    ));
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        children![(
            StateScoped(GameState::Playing),
            Node {
                width: Val::Px(300.0),
                height: Val::Px(30.0),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            BorderColor(css::WHITE.into()),
            children![(
                EnergyFill,
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(css::GREEN.into()),
            )],
        )],
    ));
}

fn update_energy_display(
    ship: Single<(&Ship, &Transform)>,
    mut energy: Single<&mut Text, With<EnergyText>>,
) {
    energy.0 = format!(
        "Energy: {:.1}\nDistance: {:.0}",
        ship.0.energy,
        ship.1.translation.length()
    );
}

fn update_energy(
    ship: Single<&Ship>,
    bar: Single<(&mut Node, &mut BackgroundColor), With<EnergyFill>>,
    scaling: Res<Scaling>,
) {
    let fraction = ship.energy / scaling.max_battery;
    let (mut bar, mut background) = bar.into_inner();
    bar.width = Val::Percent(fraction * 100.0);
    background.0 = (css::GREEN.mix(&css::RED, 1.0 - fraction)).into();
}

pub struct EnergyDisplayPlugin;

impl Plugin for EnergyDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(
                Update,
                (update_energy, update_energy_display).run_if(in_state(GameRunState::Playing)),
            );
    }
}
