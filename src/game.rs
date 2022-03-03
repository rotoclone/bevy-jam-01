use std::{cmp::Ordering, collections::HashSet};

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
            .add_system(district_selection_system)
            .add_system(tile_click_system)
            .add_system(map_update_system)
            .insert_resource(SelectedDistrict(0))
            .insert_resource(STARTING_LEVEL)
            .insert_resource(Map(vec![]));
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

    /// Gets the tile with the provided coordinates, if it exists.
    fn get(&self, coords: &Coordinates) -> &MapTile {
        &self.0[coords.y][coords.x]
    }

    /// Gets the tile with the provided coordinates mutably, if it exists.
    fn get_mut(&mut self, coords: &Coordinates) -> &mut MapTile {
        &mut self.0[coords.y][coords.x]
    }

    /// Calculates results for all the districts
    fn get_district_results(&self, num_districts: u8) -> Vec<DistrictResult> {
        let mut results = Vec::new();
        for district_id in 0..num_districts {
            let tiles = self.get_tiles_in_district(district_id);
            let good_tiles = tiles
                .iter()
                .filter(|tile| tile.content == MapTileContent::Good)
                .count();
            let bad_tiles = tiles
                .iter()
                .filter(|tile| tile.content == MapTileContent::Bad)
                .count();
            let winner = if are_contiguous(&tiles) {
                match good_tiles.cmp(&bad_tiles) {
                    Ordering::Greater => Some(DistrictWinner::Good),
                    Ordering::Less => Some(DistrictWinner::Bad),
                    Ordering::Equal => Some(DistrictWinner::Tie),
                }
            } else {
                None
            };

            results.push(DistrictResult {
                size: good_tiles + bad_tiles,
                winner,
            });
        }

        results
    }

    /// Gets all the tiles in the provided district
    fn get_tiles_in_district(&self, district_id: u8) -> Vec<&MapTile> {
        let mut tiles = Vec::new();
        for row in self.0.iter() {
            for tile in row {
                if tile.district_id == Some(district_id) {
                    tiles.push(tile)
                }
            }
        }

        tiles
    }
}

/// Determines if the provided tiles are contiguous
fn are_contiguous(tiles: &[&MapTile]) -> bool {
    match tiles.first() {
        Some(tile) => {
            tile.find_contiguous_tiles(tiles, HashSet::<&MapTile>::new())
                == tiles.iter().cloned().collect::<HashSet<&MapTile>>()
        }
        None => false,
    }
}

struct DistrictResult {
    size: usize,
    winner: Option<DistrictWinner>,
}

enum DistrictWinner {
    Good,
    Bad,
    Tie,
}

struct Level {
    /// The number of districts required
    districts: u8,
    /// What percentage of the population will vote with the good party
    good_pct: f32,
    /// What percentage of the map will be populated
    populated_pct: f32,
    /// The size of the x and y dimensions of the map
    map_size: usize,
}

#[derive(Hash, PartialEq, Eq)]
struct MapTile {
    coords: Coordinates,
    content: MapTileContent,
    district_id: Option<u8>,
}

impl MapTile {
    /// Determines whether this tile is orthogonally adjacent to the provided tile
    fn adjacent_to(&self, other: &MapTile) -> bool {
        ((self.coords.x == other.coords.x + 1
            || (other.coords.x > 0 && self.coords.x == other.coords.x - 1))
            && self.coords.y == other.coords.y)
            || ((self.coords.y == other.coords.y + 1
                || (other.coords.y > 0 && self.coords.y == other.coords.y - 1))
                && self.coords.x == other.coords.x)
    }

    /// Determines which of the provided tiles are orthogonally adjacent to this tile
    fn find_adjacent_tiles<'a>(&self, tiles: &'a [&MapTile]) -> Vec<&&'a MapTile> {
        tiles.iter().filter(|tile| tile.adjacent_to(self)).collect()
    }

    /// Determines which of the provided tiles are contiguous with this tile (i.e. transitively adjacent to it)
    fn find_contiguous_tiles<'a>(
        &'a self,
        tiles: &'a [&'a MapTile],
        mut checked_tiles: HashSet<&'a MapTile>,
    ) -> HashSet<&'a MapTile> {
        if checked_tiles.contains(&self) {
            return checked_tiles;
        }
        checked_tiles.insert(self);

        for tile in self.find_adjacent_tiles(tiles) {
            checked_tiles = tile.find_contiguous_tiles(tiles, checked_tiles);
        }

        checked_tiles
    }
}

#[derive(PartialEq, Eq, Hash)]
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

    fn new_good(x: usize, y: usize) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Good)
    }

    fn new_bad(x: usize, y: usize) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Bad)
    }

    fn new_empty(x: usize, y: usize) -> Self {
        MapTile::with_content(Coordinates { x, y }, MapTileContent::Empty)
    }

    fn color(&self, colors: &Colors) -> Color {
        match self.content {
            MapTileContent::Good => colors.good_faded,
            MapTileContent::Bad => colors.bad_faded,
            MapTileContent::Empty => EMPTY_TILE_COLOR,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
struct Coordinates {
    x: usize,
    y: usize,
}

/// Sets up the main game screen.
fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    colors: Res<Colors>,
    level: Res<Level>,
) {
    // set up map
    let num_rows = level.map_size;
    let num_columns = level.map_size;
    let map = Map::generate(&level);

    // spawn map display
    let tile_spacing = 1.0;
    let tile_size = Vec3::new(15.0, 15.0, 1.0);
    let tiles_width = num_columns as f32 * (tile_size.x + tile_spacing) - tile_spacing;
    let tiles_height = num_rows as f32 * (tile_size.y + tile_spacing) - tile_spacing;
    // center the tiles
    let tiles_offset = Vec3::new(
        -(tiles_width - tile_size.x) / 2.0,
        -(tiles_height - tile_size.y) / 2.0,
        0.0,
    );
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
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
                        color: map_tile.color(&colors),
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
                .insert(map_tile.coords.clone())
                .with_children(|parent| {
                    parent
                        .spawn_bundle(Text2dBundle {
                            text: Text::with_section(
                                "",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 25.0,
                                    color: Color::GREEN,
                                },
                                Default::default(),
                            ),
                            transform: Transform {
                                translation: Vec3::new(-0.3, 0.57, 2.0),
                                scale: Vec3::new(0.045, 0.045, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(map_tile.coords.clone());
                });
        }
    }

    commands.insert_resource(map);

    // spawn district selection buttons
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

/// Handles interactions with map tiles
fn tile_click_system(
    buttons: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    selected_district: ResMut<SelectedDistrict>,
    mut map: ResMut<Map>,
    query: Query<(&Transform, &Coordinates, &Children)>,
    mut query_child: Query<&mut Text>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(pos) = cursor_position.0 {
            for (transform, coords, children) in query.iter() {
                if intersects(pos, transform) {
                    map.get_mut(coords).district_id = Some(selected_district.0);
                    for &child in children.iter() {
                        let mut text = query_child.get_mut(child).unwrap();
                        text.sections[0].value = format!("{}", selected_district.0 + 1);
                    }
                }
            }
        }
    }
}

/// Handles updating the map based on district winners
fn map_update_system(
    map: Res<Map>,
    level: Res<Level>,
    colors: Res<Colors>,
    query: Query<(&Coordinates, &Children)>,
    mut query_child: Query<&mut Text>,
) {
    let results = map.get_district_results(level.districts);
    for (coords, children) in query.iter() {
        let tile = map.get(coords);
        for &child in children.iter() {
            let mut text = query_child.get_mut(child).unwrap();
            if let Some(district_id) = tile.district_id {
                let color = match results[district_id as usize].winner {
                    Some(DistrictWinner::Good) => colors.good_regular,
                    Some(DistrictWinner::Bad) => colors.bad_regular,
                    Some(DistrictWinner::Tie) => Color::YELLOW_GREEN,
                    None => Color::GREEN,
                };
                text.sections[0].style.color = color;
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
