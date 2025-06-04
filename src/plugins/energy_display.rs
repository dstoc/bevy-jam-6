use bevy::prelude::*;

use super::ship::Ship;

#[derive(Component)]
struct EnergyText;

pub struct EnergyDisplayPlugin;

impl Plugin for EnergyDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_energy_display)
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
}

fn update_energy_display(ship: Single<&Ship>, mut energy: Single<&mut Text, With<EnergyText>>) {
    energy.0 = format!("Energy: {:.1}", ship.energy);
}
