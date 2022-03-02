use crate::*;
use rand::Rng;

const EMPTY_TILE_COLOR: Color = Color::WHITE;

#[derive(Component)]
pub struct GameComponent;

struct Map(Vec<Vec<MapTile>>);

impl Map {
    fn generate(num_rows: u32, num_columns: u32) -> Self {
        let mut rows = Vec::new();
        for y in 0..num_rows {
            let mut row = Vec::new();
            for x in 0..num_columns {
                let tile = match rand::thread_rng().gen::<f32>() {
                    r if r < 0.25 => MapTile::new_good(x, y),
                    r if r < 0.75 => MapTile::new_bad(x, y),
                    _ => MapTile::new_empty(x, y),
                };
                row.push(tile);
            }
            rows.push(row);
        }

        Map(rows)
    }
}

#[derive(Component)]
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

#[derive(Component, Clone)]
struct Coordinates {
    x: u32,
    y: u32,
}

/// Sets up the main game screen.
pub fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    good_color: Res<GoodColor>,
    bad_color: Res<BadColor>,
) {
    // set up map
    let num_rows = 50;
    let num_columns = 50;
    let map = Map::generate(num_rows, num_columns);

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
}
