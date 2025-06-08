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
        "Increase ion storage".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 10
    }
    fn apply(&self, _level: u32, scaling: &mut Scaling) {
        scaling.max_battery += 500.0;
    }
}

struct TestUpgrade;

impl Upgrade for TestUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Test Upgrade".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, _scaling: &mut Scaling) {}
}

struct TestHiddenUpgrade;

impl Upgrade for TestHiddenUpgrade {
    fn description(&self, _level: u32, _scaling: &Scaling) -> String {
        "Test Upgrade".into()
    }
    fn cost(&self, level: u32) -> u32 {
        level * 2
    }
    fn apply(&self, _level: u32, _scaling: &mut Scaling) {}
    fn hidden(&self, _scaling: &Scaling, _data: &GameData) -> bool {
        true
    }
}

const UPGRADES: &[&dyn Upgrade] = &[
    &BatteryUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestUpgrade,
    &TestHiddenUpgrade,
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
    mut scaling: ResMut<Scaling>,
    mut data: ResMut<GameData>,
    mut levels: ResMut<UpgradeLevels>,
    commands: Commands,
    upgrade_container: Single<Entity, With<UpgradeContainer>>,
) {
    let upgrade_state = upgrade_state.get(trigger.target()).unwrap();
    let level: u32 = levels.levels[upgrade_state.index];
    let next_level = level + 1;
    levels.levels[upgrade_state.index] += 1;
    let upgrade = UPGRADES[upgrade_state.index];
    upgrade.apply(next_level, scaling.as_mut());
    data.network_credits -= upgrade.cost(next_level);

    let upgrades = summarise_upgrades(&scaling, &data, &levels);
    rebuild_upgrades(commands, *upgrade_container, upgrades);
}

fn summarise_upgrades(
    scaling: &Scaling,
    data: &GameData,
    levels: &UpgradeLevels,
) -> Vec<UpgradeState> {
    UPGRADES
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
        .collect()
}

fn rebuild_upgrades(mut commands: Commands, parent: Entity, upgrades: Vec<UpgradeState>) {
    commands.entity(parent).despawn_related::<Children>();

    commands.entity(parent).with_children(|parent| {
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
                    width: Val::Px(300.0),
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

fn setup(
    mut commands: Commands,
    scaling: Res<Scaling>,
    data: Res<GameData>,
    levels: Res<UpgradeLevels>,
) {
    let upgrades = summarise_upgrades(&scaling, &data, &levels);
    let mut upgrade_container = None;
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
            children![(
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
            )],
        ))
        .with_children(|parent| {
            upgrade_container = Some(
                parent
                    .spawn((
                        UpgradeContainer,
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: Val::Px(10.0),
                            column_gap: Val::Px(10.0),
                            height: Val::Px(600.0),
                            ..default()
                        },
                    ))
                    .id(),
            );
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

    // TODO: I couldn't figure out a way to be able to call this helper from
    // inside `with_children`.
    rebuild_upgrades(commands, upgrade_container.unwrap(), upgrades);
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
        app.add_systems(
            Update,
            update_link_text.run_if(in_state(GameState::Shop).and(resource_changed::<GameData>)),
        );
    }
}
