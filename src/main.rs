use bevy::{
    app::AppExit,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};

mod menu;
use menu::*;

mod game;
use game::*;

mod game_over;
use game_over::*;

const DEV_MODE: bool = true;

const NORMAL_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

const COLOR_1: Color = Color::BLUE;
const COLOR_2: Color = Color::RED;

pub struct GoodColor(Color);
pub struct BadColor(Color);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Menu,
    Game,
    GameOver,
}

#[derive(Component)]
struct ExitButton;

/// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_components<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

type InteractedButtonTuple = (Changed<Interaction>, With<Button>);

/// Handles changing button colors when they're interacted with.
fn button_color_system(
    mut interaction_query: Query<(&Interaction, &mut UiColor), InteractedButtonTuple>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        *color = match *interaction {
            Interaction::Clicked => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

type InteractedExitButtonTuple = (Changed<Interaction>, With<ExitButton>);

/// Handles interactions with the exit button.
fn exit_button_system(
    mut app_exit_events: EventWriter<AppExit>,
    interaction_query: Query<&Interaction, InteractedExitButtonTuple>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            app_exit_events.send(AppExit);
        }
    }
}

/// Handles showing the world inspector.
fn world_inspector_system(
    keyboard: Res<Input<KeyCode>>,
    mut inspector_params: ResMut<WorldInspectorParams>,
) {
    if keyboard.pressed(KeyCode::Equals) {
        inspector_params.enabled = true;
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Redistricting".to_string(),
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .insert_resource(GoodColor(COLOR_1))
        .insert_resource(BadColor(COLOR_2))
        .add_state(GameState::Menu)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(button_color_system)
        .add_system(start_button_system)
        .add_system(exit_button_system)
        .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(menu_setup))
        .add_system_set(
            SystemSet::on_exit(GameState::Menu).with_system(despawn_components::<MenuComponent>),
        )
        .add_system_set(SystemSet::on_enter(GameState::Game).with_system(game_setup))
        .add_system_set(
            SystemSet::on_exit(GameState::Game).with_system(despawn_components::<GameComponent>),
        )
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(game_over_setup))
        .add_system_set(
            SystemSet::on_exit(GameState::GameOver)
                .with_system(despawn_components::<GameOverComponent>),
        )
        .add_plugins(DefaultPlugins);

    if DEV_MODE {
        app.add_system(world_inspector_system)
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(WorldInspectorPlugin::new())
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..Default::default()
            });
    }

    app.run();
}
