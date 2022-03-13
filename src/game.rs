use std::{cmp::Ordering, collections::HashSet};

use crate::*;
use rand::Rng;

const EMPTY_TILE_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const EMPTY_TILE_COLOR_FADED: Color = Color::rgb(0.8, 0.8, 0.8);
const BORDER_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
const MIN_DISTRICTS: u8 = 3;
const MAX_DISTRICTS: u8 = 9;
const MIN_GOOD_PCT: f32 = 0.25;
const MAX_POPULATED_PCT: f32 = 0.9;
const MAX_MAP_SIZE: usize = 20;
const STARTING_LEVEL: Level = Level {
    districts: 3,
    good_pct: 0.5,
    populated_pct: 0.7,
    map_size: 8,
    min_district_size: 18,
    max_district_size: 20,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(game_setup))
            .add_system_set(
                SystemSet::on_exit(GameState::Game)
                    .with_system(despawn_components_system::<GameComponent>),
            )
            .add_system(district_selection_system)
            .add_system(tile_click_system)
            .add_system(map_update_system)
            .add_system(border_system)
            .add_system(district_info_system)
            .add_system(solution_system)
            .add_system(confirm_button_visibility_system)
            .add_system(confirm_button_system)
            .insert_resource(SelectedDistrict(0))
            .insert_resource(Solved(false))
            .insert_resource(Score(0))
            .insert_resource(STARTING_LEVEL)
            .insert_resource(Map {
                tiles: vec![],
                num_non_empty_tiles: 0,
            });
    }
}

#[derive(Component)]
struct GameComponent;

#[derive(Component)]
struct DistrictSelector(u8);

#[derive(Component)]
struct ConfirmButton;

#[derive(Component)]
struct ConfirmButtonParent;

#[derive(Component)]
enum Border {
    Top,
    Bottom,
    Left,
    Right,
}

struct SelectedDistrict(u8);

struct Solved(bool);

struct Score(u32);

struct Map {
    tiles: Vec<Vec<MapTile>>,
    num_non_empty_tiles: usize,
}

impl Map {
    fn generate(level: &mut Level) -> Self {
        let mut num_non_empty_tiles = 0;
        let mut num_good_tiles = 0;
        let mut rows = Vec::new();
        for y in 0..level.map_size {
            let mut row = Vec::new();
            for x in 0..level.map_size {
                let tile = if rand::thread_rng().gen::<f32>() <= level.populated_pct {
                    num_non_empty_tiles += 1;
                    match rand::thread_rng().gen::<f32>() {
                        r if r <= level.good_pct => {
                            num_good_tiles += 1;
                            MapTile::new_good(x, y)
                        }
                        _ => MapTile::new_bad(x, y),
                    }
                } else {
                    MapTile::new_empty(x, y)
                };
                row.push(tile);
            }
            rows.push(row);
        }

        let mut map = Map {
            tiles: rows,
            num_non_empty_tiles,
        };
        level.set_district_sizes(num_non_empty_tiles);

        // adjust the generated level so the number of good tiles is within a reasonable range of the prescribed percentage, and also there are enough good tiles to be able to win
        let min_good_tiles = determine_min_good_tiles(level, num_non_empty_tiles);

        let min_good_tile_fraction = level.good_pct;
        let max_good_tile_fraction = level.good_pct * 1.1;
        let mut good_tile_fraction = num_good_tiles as f32 / num_non_empty_tiles as f32;
        while good_tile_fraction > max_good_tile_fraction {
            // there are too many good tiles, turn one to the dark side
            let coords = map.find_random_coords_with_content(MapTileContent::Good);
            map.get_mut(&coords).content = MapTileContent::Bad;
            num_good_tiles -= 1;
            good_tile_fraction = num_good_tiles as f32 / num_non_empty_tiles as f32;
        }

        while good_tile_fraction < min_good_tile_fraction || num_good_tiles < min_good_tiles {
            // there are not enough good tiles, wololo
            let coords = map.find_random_coords_with_content(MapTileContent::Bad);
            map.get_mut(&coords).content = MapTileContent::Good;
            num_good_tiles += 1;
            good_tile_fraction = num_good_tiles as f32 / num_non_empty_tiles as f32;
        }

        println!("minimum good tiles: {min_good_tiles}, actual good tiles: {num_good_tiles}"); //TODO remove

        map
    }

    /// Mutably finds the coordinates of a random tile with the provided content
    fn find_random_coords_with_content(&self, content: MapTileContent) -> Coordinates {
        let tiles = self.get_tiles_with_content(content);
        tiles[rand::thread_rng().gen_range(0..tiles.len())]
            .coords
            .clone()
    }

    /// Gets the tile with the provided coordinates, if it exists.
    fn get(&self, coords: &Coordinates) -> &MapTile {
        &self.tiles[coords.y][coords.x]
    }

    /// Gets the tile with the provided coordinates mutably, if it exists.
    fn get_mut(&mut self, coords: &Coordinates) -> &mut MapTile {
        &mut self.tiles[coords.y][coords.x]
    }

    /// Gets the tile one space up from the provided coordinates, if it exists
    fn get_up(&self, coords: &Coordinates) -> Option<&MapTile> {
        if coords.y == 0 {
            None
        } else {
            Some(self.get(&Coordinates {
                x: coords.x,
                y: coords.y - 1,
            }))
        }
    }

    /// Gets the tile one space down from the provided coordinates, if it exists
    fn get_down(&self, coords: &Coordinates) -> Option<&MapTile> {
        if coords.y >= (self.tiles.len() - 1) {
            None
        } else {
            Some(self.get(&Coordinates {
                x: coords.x,
                y: coords.y + 1,
            }))
        }
    }

    /// Gets the tile one space left from the provided coordinates, if it exists
    fn get_left(&self, coords: &Coordinates) -> Option<&MapTile> {
        if coords.x == 0 {
            None
        } else {
            Some(self.get(&Coordinates {
                x: coords.x - 1,
                y: coords.y,
            }))
        }
    }

    /// Gets the tile one space right from the provided coordinates, if it exists
    fn get_right(&self, coords: &Coordinates) -> Option<&MapTile> {
        if coords.x >= (self.tiles[0].len() - 1) {
            None
        } else {
            Some(self.get(&Coordinates {
                x: coords.x + 1,
                y: coords.y,
            }))
        }
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

    /// Gets all the tiles for which the provided filter function returns true
    fn get_tiles_matching<F>(&self, filter_fn: F) -> Vec<&MapTile>
    where
        F: Fn(&MapTile) -> bool,
    {
        let mut tiles = Vec::new();
        for row in self.tiles.iter() {
            for tile in row {
                if filter_fn(tile) {
                    tiles.push(tile)
                }
            }
        }

        tiles
    }

    /// Gets all the tiles in the provided district
    fn get_tiles_in_district(&self, district_id: u8) -> Vec<&MapTile> {
        self.get_tiles_matching(|tile| tile.district_id == Some(district_id))
    }

    /// Gets all the tiles with the provided content
    fn get_tiles_with_content(&self, content: MapTileContent) -> Vec<&MapTile> {
        self.get_tiles_matching(|tile| tile.content == content)
    }
}

/// Determines the minimum number of good tiles needed for a level to not be impossible
fn determine_min_good_tiles(level: &Level, num_non_empty_tiles: usize) -> usize {
    let min_good_tiles_per_good_district = (level.min_district_size / 2) + 1;
    let min_districts_to_win = (level.districts / 2) + 1;
    let mut district_sizes = Vec::new();
    for i in 0..level.districts {
        if i < min_districts_to_win {
            district_sizes.push(level.min_district_size);
        } else {
            district_sizes.push(level.max_district_size);
        }
    }

    let total_district_sizes = district_sizes.iter().sum::<usize>();
    let extra_good_tiles_needed = if num_non_empty_tiles > total_district_sizes {
        let extra_tiles = num_non_empty_tiles - total_district_sizes;
        (extra_tiles / 2) + 1
    } else {
        0
    };

    (min_good_tiles_per_good_district * min_districts_to_win as usize) + extra_good_tiles_needed
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

impl DistrictResult {
    fn validity(&self, level: &Level) -> DistrictValidity {
        if self.size < level.min_district_size {
            DistrictValidity::TooSmall
        } else if self.size > level.max_district_size {
            DistrictValidity::TooBig
        } else if self.winner.is_none() {
            DistrictValidity::NonContiguous
        } else {
            DistrictValidity::Valid
        }
    }
}

#[derive(PartialEq, Eq)]
enum DistrictValidity {
    TooSmall,
    TooBig,
    NonContiguous,
    Valid,
}

#[derive(PartialEq, Eq)]
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
    /// The minimum population in a district
    min_district_size: usize,
    /// The maximum population in a district
    max_district_size: usize,
}

impl Level {
    /// Sets min and max district sizes based on the provided number of non-empty tiles on the map
    fn set_district_sizes(&mut self, num_non_empty_tiles: usize) {
        let avg_district_size = num_non_empty_tiles as f32 / self.districts as f32;
        self.min_district_size = (avg_district_size * 0.95).round() as usize;
        self.max_district_size = (avg_district_size * 1.05).round() as usize;
    }
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
            MapTileContent::Good => colors.good_regular,
            MapTileContent::Bad => colors.bad_regular,
            MapTileContent::Empty => EMPTY_TILE_COLOR,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
struct Coordinates {
    x: usize,
    y: usize,
}

fn set_up_game(
    commands: &mut Commands,
    asset_server: &AssetServer,
    colors: &Colors,
    level: &mut Level,
    score: &Score,
) {
    // set up map
    let num_rows = level.map_size;
    let num_columns = level.map_size;
    let map = Map::generate(level);

    // spawn map display
    let tile_spacing = 1.0;
    let tiles_width = 470.0;
    let tiles_height = 470.0;
    let tile_size = Vec3::new(
        ((tiles_width + tile_spacing) / num_columns as f32) - tile_spacing,
        ((tiles_height + tile_spacing) / num_rows as f32) - tile_spacing,
        1.0,
    );
    // center the tiles
    let tiles_offset = Vec3::new(
        -(tiles_width - tile_size.x) / 2.0,
        -(tiles_height - tile_size.y) / 2.0,
        0.0,
    );
    let font = asset_server.load(MAIN_FONT);
    let mono_font = asset_server.load(MONO_FONT);
    for (row_idx, map_row) in map.tiles.iter().rev().enumerate() {
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
                        color: map_tile.color(colors),
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
                                    font: mono_font.clone(),
                                    font_size: 125.0,
                                    color: Color::GREEN,
                                },
                                Default::default(),
                            ),
                            transform: Transform {
                                translation: Vec3::new(-0.3, 0.6, 2.0),
                                scale: Vec3::new(0.01, 0.01, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(map_tile.coords.clone());

                    let border_thickness = 0.2;
                    let border_length = 1.2;
                    let border_offset = 0.5;

                    // top border
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: BORDER_COLOR,
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(0.0, border_offset, 3.0),
                                scale: Vec3::new(border_length, border_thickness, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Border::Top);

                    // bottom border
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: BORDER_COLOR,
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(0.0, -border_offset, 3.0),
                                scale: Vec3::new(border_length, border_thickness, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Border::Bottom);

                    // left border
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: BORDER_COLOR,
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(-border_offset, 0.0, 3.0),
                                scale: Vec3::new(border_thickness, border_length, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Border::Left);

                    // right border
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: BORDER_COLOR,
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(border_offset, 0.0, 3.0),
                                scale: Vec3::new(border_thickness, border_length, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Border::Right);
                });
        }
    }

    // spawn district selection buttons
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(3.0),
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
                                    font: mono_font.clone(),
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

    //spawn score display and level info
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(25.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(3.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .insert(GameComponent)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    format!("Years in power: {}", score.0),
                    TextStyle {
                        font: font.clone(),
                        font_size: 20.0,
                        color: Color::SEA_GREEN,
                    },
                    Default::default(),
                ),
                style: Style {
                    margin: Rect {
                        bottom: Val::Px(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            let num_good_tiles = map.get_tiles_with_content(MapTileContent::Good).len();

            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    format!(
                        "You are in the {} party.\n{}% of voters will vote for your party.\nDraw {} districts with {} to {} voters each.",
                        colors.good_color_name,
                        ((num_good_tiles as f32 / map.num_non_empty_tiles as f32) * 100.0).round() as u32,
                        level.districts,
                        level.min_district_size,
                        level.max_district_size,
                    ),
                    TextStyle {
                        font: font.clone(),
                        font_size: 30.0,
                        color: Color::SEA_GREEN,
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    }
                ),
                style: Style {
                    justify_content: JustifyContent::FlexEnd,
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    commands.insert_resource(map);
}

/// Sets up the main game screen.
fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    colors: Res<Colors>,
    mut level: ResMut<Level>,
    score: Res<Score>,
) {
    set_up_game(&mut commands, &asset_server, &colors, &mut level, &score);
}

/// Handles interactions with map tiles
fn tile_click_system(
    buttons: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    selected_district: ResMut<SelectedDistrict>,
    colors: Res<Colors>,
    mut map: ResMut<Map>,
    mut query: Query<(&Transform, &Coordinates, &mut Sprite, &Children)>,
    mut query_child: Query<&mut Text>,
) {
    if buttons.pressed(MouseButton::Left) || buttons.pressed(MouseButton::Right) {
        if let Some(pos) = cursor_position.0 {
            for (transform, coords, mut sprite, children) in query.iter_mut() {
                if intersects(pos, transform) {
                    let mut tile = map.get_mut(coords);
                    if buttons.pressed(MouseButton::Left) {
                        tile.district_id = Some(selected_district.0);
                        sprite.color = match tile.content {
                            MapTileContent::Good => colors.good_faded,
                            MapTileContent::Bad => colors.bad_faded,
                            MapTileContent::Empty => EMPTY_TILE_COLOR_FADED,
                        };
                        for &child in children.iter() {
                            if let Ok(mut text) = query_child.get_mut(child) {
                                text.sections[0].value = format!("{}", selected_district.0 + 1);
                            }
                        }
                    } else if buttons.pressed(MouseButton::Right) {
                        tile.district_id = None;
                        sprite.color = tile.color(&colors);
                        for &child in children.iter() {
                            if let Ok(mut text) = query_child.get_mut(child) {
                                text.sections[0].value = "".to_string();
                            }
                        }
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
            if let Ok(mut text) = query_child.get_mut(child) {
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

/// Handles showing district borders
fn border_system(
    map: Res<Map>,
    query: Query<(&Coordinates, &Children)>,
    mut query_child: Query<(&Border, &mut Sprite)>,
) {
    for (coords, children) in query.iter() {
        let tile = map.get(coords);

        let needs_top_border = if let Some(up_tile) = map.get_up(coords) {
            tile.district_id != up_tile.district_id
        } else {
            tile.district_id.is_some()
        };

        let needs_bottom_border = if let Some(down_tile) = map.get_down(coords) {
            tile.district_id != down_tile.district_id
        } else {
            tile.district_id.is_some()
        };

        let needs_left_border = if let Some(left_tile) = map.get_left(coords) {
            tile.district_id != left_tile.district_id
        } else {
            tile.district_id.is_some()
        };

        let needs_right_border = if let Some(right_tile) = map.get_right(coords) {
            tile.district_id != right_tile.district_id
        } else {
            tile.district_id.is_some()
        };

        for &child in children.iter() {
            if let Ok((border, mut sprite)) = query_child.get_mut(child) {
                if matches!(border, Border::Top) {
                    if needs_top_border {
                        sprite.color.set_a(1.0);
                    } else {
                        sprite.color.set_a(0.0);
                    }
                }

                if matches!(border, Border::Bottom) {
                    if needs_bottom_border {
                        sprite.color.set_a(1.0);
                    } else {
                        sprite.color.set_a(0.0);
                    }
                }

                if matches!(border, Border::Left) {
                    if needs_left_border {
                        sprite.color.set_a(1.0);
                    } else {
                        sprite.color.set_a(0.0);
                    }
                }

                if matches!(border, Border::Right) {
                    if needs_right_border {
                        sprite.color.set_a(1.0);
                    } else {
                        sprite.color.set_a(0.0);
                    }
                }
            }
        }
    }
}

/// Handles displaying info about the current districts
fn district_info_system(
    map: Res<Map>,
    level: Res<Level>,
    button_query: Query<(&DistrictSelector, &Children)>,
    mut query_child: Query<&mut Text>,
) {
    let results = map.get_district_results(level.districts);
    for (district_selector, children) in button_query.iter() {
        for &child in children.iter() {
            if let Ok(mut text) = query_child.get_mut(child) {
                let result = &results[district_selector.0 as usize];
                let validity_text = match result.validity(&level) {
                    DistrictValidity::TooBig => " [too big]",
                    DistrictValidity::TooSmall => " [too small]",
                    DistrictValidity::NonContiguous => " [non-contiguous]",
                    DistrictValidity::Valid => match result.winner {
                        Some(DistrictWinner::Good) => " [win]",
                        Some(DistrictWinner::Bad) => " [lose]",
                        Some(DistrictWinner::Tie) => " [tie]",
                        None => " [invalid]",
                    },
                };
                text.sections[0].value = format!(
                    "District {} ({}){validity_text}",
                    district_selector.0 + 1,
                    result.size
                );
            }
        }
    }
}

/// Handles determining whether the level is solved
fn solution_system(mut solved: ResMut<Solved>, map: Res<Map>, level: Res<Level>) {
    let results = map.get_district_results(level.districts);

    // make sure all districts are the right size and have a winner
    let any_invalid_districts = results
        .iter()
        .any(|result| result.validity(&level) != DistrictValidity::Valid);
    if any_invalid_districts {
        solved.0 = false;
        return;
    }

    // make sure all tiles are in a district
    let any_districtless_tiles = map
        .tiles
        .iter()
        .any(|row| row.iter().any(|tile| tile.district_id == None));
    if any_districtless_tiles {
        solved.0 = false;
        return;
    }

    let good_wins = results
        .iter()
        .filter(|result| result.winner == Some(DistrictWinner::Good))
        .count();
    if good_wins as f32 > (level.districts as f32 / 2.0) {
        solved.0 = true;
    } else {
        solved.0 = false;
    }
}

/// Handles showing and hiding the confirm button
fn confirm_button_visibility_system(
    solved: Res<Solved>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Style, With<ConfirmButtonParent>>,
) {
    if solved.0 {
        if query.is_empty() {
            let font = asset_server.load(MAIN_FONT);
            commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(25.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(3.0),
                            ..Default::default()
                        },
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::ColumnReverse,
                        ..Default::default()
                    },
                    color: UiColor(Color::NONE),
                    ..Default::default()
                })
                .insert(GameComponent)
                .insert(ConfirmButtonParent)
                .with_children(|parent| {
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
                        .insert(ConfirmButton)
                        .with_children(|parent| {
                            parent.spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    "Confirm",
                                    TextStyle {
                                        font,
                                        font_size: 20.0,
                                        color: Color::SEA_GREEN,
                                    },
                                    Default::default(),
                                ),
                                ..Default::default()
                            });
                        });
                });
        } else {
            for mut style in query.iter_mut() {
                style.display = Display::Flex;
            }
        }
    } else {
        for mut style in query.iter_mut() {
            style.display = Display::None;
        }
    }
}

type InteractedConfirmButtonTuple = (Changed<Interaction>, With<ConfirmButton>);

/// Handles interactions with the confirm button.
#[allow(clippy::too_many_arguments)]
fn confirm_button_system(
    mut level: ResMut<Level>,
    mut score: ResMut<Score>,
    mut solved: ResMut<Solved>,
    mut selected_district: ResMut<SelectedDistrict>,
    asset_server: Res<AssetServer>,
    colors: Res<Colors>,
    mut commands: Commands,
    interaction_query: Query<&Interaction, InteractedConfirmButtonTuple>,
    to_despawn_query: Query<Entity, With<GameComponent>>,
) {
    let mut change_level = false;
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            change_level = true;
            break;
        }
    }

    if change_level {
        score.0 += 10;
        *level = generate_next_level(&level);
        solved.0 = false;
        selected_district.0 = 0;
        despawn_components(to_despawn_query, &mut commands);
        set_up_game(&mut commands, &asset_server, &colors, &mut level, &score);
    }
}

/// Generates the next level using the previous level as a baseline
fn generate_next_level(old_level: &Level) -> Level {
    let map_size = MAX_MAP_SIZE.min(old_level.map_size + 1);
    // ensure an odd number of districts to make the game easier
    // (so you only have to win 1 more district than the bad party instead of 2)
    let tiles_per_district = 35;
    let districts = match (map_size * map_size) / tiles_per_district {
        x if x % 2 == 0 => (x + 1) as u8,
        x => x as u8,
    };
    let districts = MAX_DISTRICTS.min(MIN_DISTRICTS.max(districts));
    let populated_pct = MAX_POPULATED_PCT.min(old_level.populated_pct * 1.05);
    let avg_district_size = (map_size as f32 * map_size as f32 * populated_pct) / districts as f32;
    Level {
        districts,
        good_pct: MIN_GOOD_PCT.max(old_level.good_pct * 0.9),
        populated_pct,
        map_size,
        min_district_size: (avg_district_size * 0.95).round() as usize,
        max_district_size: (avg_district_size * 1.05).round() as usize,
    }
}
