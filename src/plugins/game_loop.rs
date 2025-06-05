use std::time::Duration;

use bevy::{prelude::*, state::state::FreelyMutableState};
use bevy_tweening::{Animator, Tween, TweenCompleted, lens::UiBackgroundColorLens};

use crate::{AppState, GameRunState, GameState};

use super::ship::Ship;

pub struct GameLoopPlugin;

fn setup_run(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::from("Ship"),
        Ship::default(),
        StateScoped(GameState::Playing),
        Mesh2d(meshes.add(Circle::new(20.0)).into()),
        MeshMaterial2d(color_materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.8)))),
        Transform::default(),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

// TODO: refactor transition logic into a separate file
// FadeState is necessary to trigger removal of the overlay.
// If we did it directly in the observer, we can get a flash
// of the old content.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum FadeState {
    #[default]
    Fading,
    Ready,
}

#[derive(Component)]
struct FadeOut;

fn check_run(mut commands: Commands, ship: Single<&Ship>) {
    if ship.energy <= 0.0 {
        commands.set_state(GameRunState::Ending);
    }
}

#[derive(Component)]
struct FadeMarker;

struct Fade(f32, f32, EaseFunction);
const FADE_IN: Fade = Fade(1.0, 0.0, EaseFunction::CubicIn);
const FADE_OUT: Fade = Fade(0.0, 1.0, EaseFunction::CubicOut);

fn fade<S: FreelyMutableState>(
    config: Fade,
    next_state: S,
) -> impl FnMut(Commands, Option<Single<Entity, With<FadeMarker>>>) {
    move |mut commands: Commands, current: Option<Single<Entity, With<FadeMarker>>>| {
        if let Some(entity) = current {
            commands.entity(*entity).despawn();
        }
        let next_state = next_state.clone();
        commands.set_state(FadeState::Fading);
        commands
            .spawn((
                FadeMarker,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0).into()),
                Animator::new(
                    Tween::new(
                        config.2,
                        Duration::from_secs(1),
                        UiBackgroundColorLens {
                            start: Color::srgba(0.0, 0.0, 0.0, config.0),
                            end: Color::srgba(0.0, 0.0, 0.0, config.1),
                        },
                    )
                    .with_completed_event(0),
                ),
                GlobalZIndex(99),
            ))
            .observe(
                move |_trigger: Trigger<TweenCompleted>, mut commands: Commands| {
                    commands.set_state(next_state.clone());
                    commands.set_state(FadeState::Ready);
                },
            );
    }
}

fn reveal(mut commands: Commands, entity: Single<Entity, With<FadeMarker>>) {
    commands.entity(*entity).despawn();
}

impl Plugin for GameLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<FadeState>()
            .add_systems(OnEnter(GameState::Playing), setup_run)
            .add_systems(Update, check_run.run_if(in_state(GameRunState::Playing)))
            .add_systems(OnEnter(FadeState::Ready), reveal)
            .add_systems(
                OnEnter(GameRunState::Ending),
                fade(FADE_OUT, AppState::MainMenu),
            )
            .add_systems(
                OnEnter(GameRunState::Playing),
                fade(FADE_IN, GameRunState::Playing),
            );
    }
}
