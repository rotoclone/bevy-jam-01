use crate::*;
use rand::Rng;

const EMPTY_TILE_COLOR: Color = Color::WHITE;
const STARTING_LEVEL: Level = Level {
    districts: 3,
    good_pct: 0.5,
    populated_pct: 0.9,
    map_size: 30,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(game_setup))
            .add_system_set(
                SystemSet::on_exit(GameState::Game)
                    .with_system(despawn_components::<GameComponent>),
            )
            .add_system(tile_system)
            .add_system(district_selection_system)
            .insert_resource(SelectedDistrict(0))
            .insert_resource(STARTING_LEVEL);
    }
}

#[derive(Component)]
struct GameComponent;

#[derive(Component)]
struct DistrictSelector(u8);

struct SelectedDistrict(u8);

struct Map(Vec<Vec<MapTile>>);

impl Map {
    fn generate(level: &Level) -> Self {
        let mut rows = Vec::new();
        for y in 0..level.map_size {
            let mut row = Vec::new();
            for x in 0..level.map_size {
                let tile = if rand::thread_rng().gen::<f32>() <= level.populated_pct {
                    match rand::thread_rng().gen::<f32>() {
                        r if r <= level.good_pct => MapTile::new_good(x, y),
                        _ => MapTile::new_bad(x, y),
                    }
                } else {
                    MapTile::new_empty(x, y)
                };
                row.push(tile);
            }
            rows.push(row);
        }

        Map(rows)
    }
}

struct Level {
    /// The number of districts required
    districts: u8,
    /// What percentage of the population will vote with the good party
    good_pct: f32,
    /// What percentage of the map will be populated
    populated_pct: f32,
    /// The size of the x and y dimensions of the map
    map_size: u32,
}

struct MapTile {
    coords: Coordinates,
    content: MapTileContent,
    district_id: Option<u8>,
}

enum MapTileContent {
    Good,
    Bad,
    Empty,
}

impl MapTile {
    fn with_content(coords: Coordinates, content: MapTileContent) -> Self {
        MapTile {
            coords,
            content,
            district_id: None,
        }
    }

    fn new_good(x: u32, y: u32) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Good)
    }

    fn new_bad(x: u32, y: u32) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Bad)
    }

    fn new_empty(x: u32, y: u32) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Empty)
    }

    fn color(&self, good_color: Color, bad_color: Color) -> Color {
        match self.content {
            MapTileContent::Good => good_color,
            MapTileContent::Bad => bad_color,
            MapTileContent::Empty => EMPTY_TILE_COLOR,
        }
    }
}

#[derive(Component, Clone, Debug)]
struct Coordinates {
    x: u32,
    y: u32,
}

/// Sets up the main game screen.
fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    good_color: Res<GoodColor>,
    bad_color: Res<BadColor>,
    level: Res<Level>,
) {
    // set up map
    let num_rows = level.map_size;
    let num_columns = level.map_size;
    let map = Map::generate(&level);

    // spawn map display
    let tile_spacing = 1.0;
    let tile_size = Vec3::new(10.0, 10.0, 1.0);
    let tiles_width = num_columns as f32 * (tile_size.x + tile_spacing) - tile_spacing;
    let tiles_height = num_rows as f32 * (tile_size.y + tile_spacing) - tile_spacing;
    // center the tiles
    let tiles_offset = Vec3::new(
        -(tiles_width - tile_size.x) / 2.0,
        -(tiles_height - tile_size.y) / 2.0,
        0.0,
    );
    for (row_idx, map_row) in map.0.iter().rev().enumerate() {
        let y_position = row_idx as f32 * (tile_size.y + tile_spacing);
        for (column_idx, map_tile) in map_row.iter().enumerate() {
            let tile_position = Vec3::new(
                column_idx as f32 * (tile_size.x + tile_spacing),
                y_position,
                0.0,
            ) + tiles_offset;
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: map_tile.color(good_color.0, bad_color.0),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: tile_position,
                        scale: tile_size,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(GameComponent)
                .insert(map_tile.coords.clone());
        }
    }

    commands.insert_resource(map);

    // spawn district selection buttons
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(0.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .insert(GameComponent)
        .with_children(|parent| {
            for district_id in 0..level.districts {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(100.0), Val::Px(50.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: Rect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..Default::default()
                    })
                    .insert(DistrictSelector(district_id))
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::with_section(
                                format!("District {}", district_id + 1),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 20.0,
                                    color: Color::SEA_GREEN,
                                },
                                Default::default(),
                            ),
                            ..Default::default()
                        });
                    });
            }
        });
}

/// Handles interactions with map tiles.
fn tile_system(
    buttons: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    mut commands: Commands,
    mut query: Query<(&Transform, &Coordinates, &mut Sprite)>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(pos) = cursor_position.0 {
            for (transform, coords, mut sprite) in query.iter_mut() {
                if intersects(pos, transform) {
                    sprite.color = Color::CYAN; //TODO
                    println!("u clicked {:?}", coords); //TODO
                }
            }
        }
    }
}

/// Determines whether a point intersects a transform
fn intersects(point: Vec2, transform: &Transform) -> bool {
    point.x >= transform.translation.x - (transform.scale.x / 2.0) - 1.0
        && point.x <= transform.translation.x + (transform.scale.x / 2.0) + 1.0
        && point.y >= transform.translation.y - (transform.scale.y / 2.0) - 1.0
        && point.y <= transform.translation.y + (transform.scale.y / 2.0) + 1.0
}

/// Handles selecting which district to paint
fn district_selection_system(
    mut selected_district: ResMut<SelectedDistrict>,
    interaction_query: Query<(&Interaction, &DistrictSelector), Changed<Interaction>>,
    mut button_query: Query<(&DistrictSelector, &mut UiColor)>,
) {
    for (interaction, district_selector) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            selected_district.0 = district_selector.0;
        }
    }

    for (district_selector, mut color) in button_query.iter_mut() {
        if selected_district.0 == district_selector.0 {
            *color = Color::WHITE.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}
