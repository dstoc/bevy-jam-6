use bevy::{color::palettes::css, prelude::*};

use super::{scaling::Scaling, ship::Ship};

#[derive(Component)]
struct EnergyText;

#[derive(Component)]
struct EnergyFill;

pub struct EnergyDisplayPlugin;

impl Plugin for EnergyDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_energy_display)
            .add_systems(Update, update_energy)
            .add_systems(Update, update_energy_display);
    }
}

fn setup_energy_display(mut commands: Commands) {
    commands.spawn((
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
    commands
        .spawn((
            Name::from("Energy Progressbar"),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(30.0),
                right: Val::Px(5.0),
                width: Val::Px(200.0),
                height: Val::Px(30.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(css::GRAY.into()),
            BorderColor(css::WHITE.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                EnergyFill,
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(css::GREEN.into()),
            ));
        });
}

fn update_energy_display(ship: Single<&Ship>, mut energy: Single<&mut Text, With<EnergyText>>) {
    energy.0 = format!("Energy: {:.1}", ship.energy);
}

fn update_energy(
    ship: Single<&Ship>,
    mut bar: Single<&mut Node, With<EnergyFill>>,
    scaling: Res<Scaling>,
) {
    bar.width = Val::Percent(ship.energy / scaling.max_battery * 100.0);
}
