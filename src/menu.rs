use crate::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Menu).with_system(menu_setup))
            .add_system_set(
                SystemSet::on_exit(GameState::Menu)
                    .with_system(despawn_components_system::<MenuComponent>),
            )
            .add_system(start_button_system);
    }
}

#[derive(Component)]
struct MenuComponent;

#[derive(Component)]
struct StartButton(Party);

enum Party {
    Red,
    Blue,
}

/// Sets up the main menu screen.
fn menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // title text
    let font = asset_server.load(MAIN_FONT);
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(0.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .insert(MenuComponent)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Redistricting".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 70.0,
                            color: Color::WHITE,
                        },
                    }],
                    alignment: TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        vertical: VerticalAlign::Center,
                    },
                },
                style: Style {
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    // start button
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(0.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .insert(MenuComponent)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(15.0)),
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(StartButton(Party::Red))
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Join the\nred party",
                            TextStyle {
                                font: font.clone(),
                                font_size: 40.0,
                                color: Color::SEA_GREEN,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                });

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(15.0)),
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(StartButton(Party::Blue))
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Join the\nblue party",
                            TextStyle {
                                font: font.clone(),
                                font_size: 40.0,
                                color: Color::SEA_GREEN,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                });
        });
}

/// Handles interactions with the start buttons.
fn start_button_system(
    mut game_state: ResMut<State<GameState>>,
    mut colors: ResMut<Colors>,
    interaction_query: Query<(&Interaction, &StartButton), Changed<Interaction>>,
) {
    for (interaction, start_button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            *colors = match start_button.0 {
                Party::Red => Colors {
                    good_color_name: "red".to_string(),
                    good_regular: RED,
                    good_faded: RED_FADED,
                    bad_regular: BLUE,
                    bad_faded: BLUE_FADED,
                },
                Party::Blue => Colors {
                    good_color_name: "blue".to_string(),
                    good_regular: BLUE,
                    good_faded: BLUE_FADED,
                    bad_regular: RED,
                    bad_faded: RED_FADED,
                },
            };
            game_state.set(GameState::Game).unwrap();
        }
    }
}
