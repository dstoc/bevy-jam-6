use bevy::{color::palettes::css, prelude::*};

use crate::{GameRunState, GameState};

use super::{scaling::Scaling, ship::Ship};

#[derive(Component)]
struct EnergyText;

#[derive(Component)]
struct BatteryFill;

#[derive(Component)]
struct CapacitorFill;

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
        #[cfg(not(debug_assertions))]
        Visibility::Hidden,
    ));
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(15.0)),
            flex_direction: FlexDirection::Row,
            // column_gap: Val::Px(15.0),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            (
                StateScoped(GameState::Playing),
                Node {
                    height: Val::Px(30.0),
                    // border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                BorderColor(css::WHITE.into()),
                children![(
                    BatteryFill,
                    Node {
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(css::GREEN.into()),
                )],
            ),
            (
                StateScoped(GameState::Playing),
                Node {
                    // width: Val::Px(300.0),
                    height: Val::Px(30.0),
                    // border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                BorderColor(css::WHITE.into()),
                children![(
                    CapacitorFill,
                    Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(css::DARK_ORANGE.into()),
                )],
            )
        ],
    ));
}

fn update_energy_text(
    ship: Single<(&Ship, &Transform)>,
    mut energy: Single<&mut Text, With<EnergyText>>,
) {
    energy.0 = format!(
        "Energy: {:.1}\nDistance: {:.0}",
        ship.0.energy,
        ship.1.translation.length()
    );
}

fn update_battery_bar(
    ship: Single<&Ship>,
    bar: Single<(&mut Node, &mut BackgroundColor), With<BatteryFill>>,
    scaling: Res<Scaling>,
) {
    let fraction = ship.energy / scaling.max_battery;
    let (mut bar, mut background) = bar.into_inner();
    bar.width = Val::Px(300.0 * fraction);
    background.0 = (css::GREEN.mix(&css::RED, 1.0 - fraction)).into();
}

fn update_capacitor_bar(
    ship: Single<&Ship>,
    mut bar: Single<&mut Node, With<CapacitorFill>>,
    scaling: Res<Scaling>,
) {
    let fraction = (ship.energy - scaling.max_battery).max(0.0) / scaling.max_capacitor;
    bar.width = Val::Px(300.0 * fraction);
}

pub struct EnergyDisplayPlugin;

impl Plugin for EnergyDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_game)
            .add_systems(
                Update,
                (update_battery_bar, update_capacitor_bar, update_energy_text)
                    .run_if(in_state(GameRunState::Playing)),
            );
    }
}
