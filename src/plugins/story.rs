use bevy::{color::palettes::css, ecs::relationship::RelatedSpawner, prelude::*};

use crate::GameState;

use super::game_loop::GameData;
use bevy::ecs::spawn::SpawnWith;

pub struct StoryPlugin;

// TODO: spanning tree/cycles
const STORIES: &[&str] = &[
    r#"In this sector of deep space, we've uncovered a vast field of Lumina nodes. Through dedicated research, we've engineered vessels powered solely by Lumina ions. These ships can establish links between nodes and draw ions when connected. To push our breakthroughs even further, we must extend and reinforce these links.

Left click applies a force in the direction of the mouse pointer.

Right click applies an automatic braking force opposite the direction of motion.

Be gentle, energy is limited."#,
    "Our ships can only draw ions when they are close to a Lumina node. We need to balance forging new links with regular stops at nodes to recharge.",
    "Our grasp of Lumina is still in its infancy. We've learned how to trigger ion bursts by linking a ship, and watched as those ions propagate outward before folding back to their origin. The effect is striking but fickle. Each trial yields unpredictable results. With deeper study, however, we're confident we can improve the consistency.",
    "It should be obvious that repeated ion bursts can temporarily shut down a Lumina node. While offline, the node will still propagate and reflect ions but won't generate more until the ship is detached and some amount of time has elapsed. Early efforts to work around this shutdown are promising, but further research is needed.",
    "In this area the ship's life-support is tied to the the distance from the point of entry. We'll need to develop further efficiencies before we can venture further.",
    "Our technology can't yet sustain unlimited links from a single Lumina node. To prevent catastrophic overloads, each node is capped at a strict connection limit. Exceeding it would result in stability collapses and a dangerous feedback loop.\n\nWe need more link data to advance our research.",
    "No further transmissions available.\n\nContinue research.",
];

#[derive(Component, Debug)]
struct StoryUi {
    links: u32,
    current: usize,
    max: usize,
}

fn shop(_trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.set_state(GameState::Shop);
}

fn next(_trigger: Trigger<Pointer<Click>>, mut story_ui: Single<&mut StoryUi>) {
    story_ui.current += 1;
}

fn prev(_trigger: Trigger<Pointer<Click>>, mut story_ui: Single<&mut StoryUi>) {
    story_ui.current -= 1;
}

fn setup(mut commands: Commands, data: Res<GameData>) {
    commands.spawn((
        Name::from("STORY"),
        StoryUi {
            links: data.last_run_network_size,
            current: (data.runs as usize - 1).min(STORIES.len() - 1),
            max: (data.runs as usize - 1).min(STORIES.len() - 1),
        },
        StateScoped(GameState::Story),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        Camera2d,
    ));
}

fn next_prev_button(text: &str, enabled: bool) -> impl Bundle {
    let color = if enabled {
        Color::WHITE
    } else {
        css::DIM_GRAY.into()
    };
    (
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(color),
        BorderRadius::all(Val::Px(5.0)),
        children![(Text::new(text), TextColor(color))],
    )
}

fn rebuild(
    mut commands: Commands,
    query: Single<(Entity, &StoryUi), Or<(Changed<StoryUi>, Added<StoryUi>)>>,
) {
    let (parent, story_ui) = query.into_inner();
    let has_next = story_ui.current < story_ui.max;
    let has_prev = story_ui.current > 0;

    commands.entity(parent).despawn_related::<Children>();
    commands.entity(parent).with_children(|parent| {
        parent.spawn((
            Text::new(format!(
                "Ship Lost...\n{:} lumina link{:} created",
                story_ui.links,
                if story_ui.links == 1 { "" } else { "s" }
            )),
            Node {
                align_self: AlignSelf::Start,
                ..default()
            },
        ));
        parent.spawn((
            Node {
                width: Val::Px(700.0),
                height: Val::Px(500.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(15.0)),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderRadius::all(Val::Px(20.0)),
            BorderColor(Color::WHITE),
            children![
                Text::new(STORIES[story_ui.current]),
                (
                    Node {
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                        let mut prev_btn = parent.spawn(next_prev_button("Prev", has_prev));
                        if has_prev {
                            prev_btn.observe(prev);
                        }
                        let mut next_btn = parent.spawn(next_prev_button("Next", has_next));
                        if has_next {
                            next_btn.observe(next);
                        }
                    }),),
                )
            ],
        ));
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::End,
                    ..default()
                },
                BorderColor(Color::WHITE),
                BorderRadius::all(Val::Px(5.0)),
                children![(Text::new("Continue"),)],
            ))
            .observe(shop);
    });
}

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Story), setup);
        app.add_systems(Update, rebuild);
    }
}
