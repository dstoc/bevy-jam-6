use bevy::prelude::*;

use crate::{AppState, GameState};

use super::{game_loop::GameData, scaling::Scaling};

pub trait Upgrade {
    fn description(&self, level: u32, scaling: &Scaling) -> String;
    fn cost(&self, level: u32) -> u32;
    fn apply(&self, _level: u32, scaling: &mut Scaling);
    fn hidden(&self, _scaling: &Scaling, _data: &GameData) -> bool {
        false
    }
}

struct BatteryUpgrade;

impl Upgrade for BatteryUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research ion storage".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 10
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.max_battery += 500.0;
    }
}

struct LuminaReflectionUpgrade;

impl Upgrade for LuminaReflectionUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina reflection".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.reflection_probability *= 1.1;
    }
    fn hidden(&self, _scaling: &Scaling, data: &GameData) -> bool {
        data.runs < 3
    }
}

struct LuminaPropagationUpgrade;

impl Upgrade for LuminaPropagationUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina propagation".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.propagation_probability *= 1.1;
    }
    fn hidden(&self, scaling: &Scaling, _data: &GameData) -> bool {
        scaling.reflection_probability < 0.6
    }
}

struct LuminaGenerationUpgrade;

impl Upgrade for LuminaGenerationUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina generation".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 3
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.generation_per_sec *= 1.15;
    }
}

struct LuminaLinksUpgrade;

impl Upgrade for LuminaLinksUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research max Lumina links".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.max_links += 1;
    }
    fn hidden(&self, scaling: &Scaling, _data: &GameData) -> bool {
        scaling.propagation_probability < 0.75
    }
}

struct LuminaCooldownUpgrade;

impl Upgrade for LuminaCooldownUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina burnout".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.lumina_cooldown_per_generation *= 0.95;
    }
    fn hidden(&self, _scaling: &Scaling, data: &GameData) -> bool {
        data.runs < 4
    }
}

struct LuminaRecoveryUpgrade;

impl Upgrade for LuminaRecoveryUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina recovery".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.lumina_resume_per_sec *= 1.05;
    }
    fn hidden(&self, scaling: &Scaling, _data: &GameData) -> bool {
        scaling.lumina_cooldown_per_generation > 0.075
    }
}

struct CapacitorUpgrade;

impl Upgrade for CapacitorUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Research Lumina capacitors".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 5
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.max_capacitor += 500.0
    }
    fn hidden(&self, scaling: &Scaling, _data: &GameData) -> bool {
        scaling.max_battery < 3000.0
    }
}

const UPGRADES: &[&dyn Upgrade] = &[
    &BatteryUpgrade,
    &LuminaReflectionUpgrade,
    &LuminaPropagationUpgrade,
    &LuminaGenerationUpgrade,
    &LuminaLinksUpgrade,
    &LuminaCooldownUpgrade,
    &LuminaRecoveryUpgrade,
    &CapacitorUpgrade,
];

#[derive(Resource)]
struct UpgradeLevels {
    levels: [u32; UPGRADES.len()],
}

#[derive(Component, Clone)]
struct UpgradeState {
    index: usize,
    description: String,
    level: u32,
    cost: u32,
    enabled: bool,
    hidden: bool,
}

#[derive(Component)]
struct UpgradeContainer;

#[derive(Component)]
struct LinksText;

fn go_next(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(GameState::Playing);
}

fn upgrade_clicked(
    trigger: Trigger<Pointer<Click>>,
    upgrade_state: Query<&UpgradeState>,
    mut commands: Commands,
    mut scaling: ResMut<Scaling>,
    mut data: ResMut<GameData>,
    mut levels: ResMut<UpgradeLevels>,
) {
    let upgrade_state = upgrade_state.get(trigger.target()).unwrap();
    let level: u32 = levels.levels[upgrade_state.index];
    let next_level = level + 1;
    levels.levels[upgrade_state.index] += 1;
    let upgrade = UPGRADES[upgrade_state.index];
    upgrade.apply(next_level, scaling.as_mut());
    data.network_credits -= upgrade.cost(next_level);

    commands.run_system_cached(rebuild_upgrades);
    commands.run_system_cached(update_link_text);
}

fn summarise_upgrades(
    scaling: &Scaling,
    data: &GameData,
    levels: &UpgradeLevels,
) -> Vec<UpgradeState> {
    let mut states: Vec<UpgradeState> = UPGRADES
        .iter()
        .enumerate()
        .map(|(index, upgrade)| UpgradeState {
            index,
            level: levels.levels[index],
            description: upgrade.description(levels.levels[index] + 1, &scaling),
            cost: upgrade.cost(levels.levels[index] + 1),
            hidden: upgrade.hidden(&scaling, &data),
            enabled: !upgrade.hidden(&scaling, &data)
                && upgrade.cost(levels.levels[index] + 1) <= data.network_credits,
        })
        .collect();
    states.sort_by(|a, b| {
        a.hidden
            .cmp(&b.hidden)
            .then(a.cost.cmp(&b.cost))
            .then_with(|| a.description.cmp(&b.description))
    });
    states
}

fn rebuild_upgrades(
    mut commands: Commands,
    parent: Single<Entity, With<UpgradeContainer>>,
    scaling: ResMut<Scaling>,
    data: ResMut<GameData>,
    levels: ResMut<UpgradeLevels>,
) {
    let upgrades = summarise_upgrades(&scaling, &data, &levels);
    commands.entity(*parent).despawn_related::<Children>();

    commands.entity(*parent).with_children(|parent| {
        for state in upgrades {
            let enabled = state.enabled;
            let color = if enabled {
                Color::WHITE
            } else {
                Color::srgb(0.2, 0.2, 0.2)
            };
            let mut builder = parent.spawn((
                Node {
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(15.0)),
                    width: Val::Px(375.0),
                    height: Val::Px(100.0),
                    ..default()
                },
                BorderColor(color),
                BorderRadius::all(Val::Px(5.0)),
                children![(
                    TextColor(color),
                    Text::new(if state.hidden {
                        "???".into()
                    } else {
                        format!(
                            "{:}\nLevel {:}\n{:} lumina links",
                            state.description,
                            state.level + 1,
                            state.cost,
                        )
                    }),
                )],
                state,
            ));
            if enabled {
                builder.observe(upgrade_clicked);
            }
        }
    });
}

fn setup(mut commands: Commands, data: Res<GameData>) {
    commands.spawn((Camera2d, StateScoped(GameState::Shop)));
    commands
        .spawn((
            StateScoped(GameState::Shop),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![
                (
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(35.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    LinksText,
                    Text::new(format!(
                        "{:} lumina link{:}",
                        data.network_credits,
                        if data.network_credits == 1 { "" } else { "s" }
                    )),
                ),
                (
                    UpgradeContainer,
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: Val::Px(10.0),
                        column_gap: Val::Px(10.0),
                        height: Val::Px(600.0),
                        ..default()
                    },
                )
            ],
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(15.0),
                        right: Val::Px(15.0),
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BorderRadius::all(Val::Px(5.0)),
                    children![(Text::new("Send ship"),)],
                ))
                .observe(go_next);
        });

    commands.run_system_cached(rebuild_upgrades);
}

fn update_link_text(mut text: Single<&mut Text, With<LinksText>>, data: Res<GameData>) {
    text.0 = format!(
        "{:} lumina link{:}",
        data.network_credits,
        if data.network_credits == 1 { "" } else { "s" }
    );
}

fn setup_game(mut commands: Commands) {
    commands.insert_resource(UpgradeLevels {
        levels: [0; UPGRADES.len()],
    });
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_game);
        app.add_systems(OnEnter(GameState::Shop), setup);
    }
}
