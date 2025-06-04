use bevy::prelude::*;

#[derive(Resource)]
pub struct Scaling {
    pub reflection_probability: f32,
    pub propagation_probability: f32,
    pub generation_per_sec: f32,
    pub max_links: usize,
    pub max_battery: f32,
    pub max_capacitor: f32,
    pub capacitor_drain_per_sec: f32,
    pub energy_extraction: f32,
    pub energy_per_force: f32,
}

impl Default for Scaling {
    fn default() -> Self {
        Scaling {
            reflection_probability: 0.5,
            propagation_probability: 0.5,
            generation_per_sec: 1.0,
            max_links: 3,
            max_battery: 2500.0,
            max_capacitor: 2500.0,
            capacitor_drain_per_sec: 1000.0,
            energy_extraction: 0.05,
            energy_per_force: 2.0,
        }
    }
}

pub struct ScalingPlugin;

impl Plugin for ScalingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scaling::default());
    }
}
