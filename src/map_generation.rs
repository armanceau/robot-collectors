use noise::{NoiseFn, Perlin};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapGenerationError {
    InvalidDimensions { width: usize, height: usize },
}

impl fmt::Display for MapGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapGenerationError::InvalidDimensions { width, height } => {
                write!(
                    f,
                    "invalid map dimensions: width={} height={} (both must be > 0)",
                    width, height
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Energy,
    Crystal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Obstacle,
    Base,
    Resource {
        kind: ResourceType,
        amount: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        self.tiles[self.index(x, y)]
    }
}

pub fn generate_map(seed: u64, width: usize, height: usize) -> Result<Map, MapGenerationError> {
    if width == 0 || height == 0 {
        return Err(MapGenerationError::InvalidDimensions { width, height });
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let perlin = Perlin::new(seed as u32);
    let mut tiles = Vec::with_capacity(width * height);

    let base_x = width / 2;
    let base_y = height / 2;
    let scale = 0.09;

    for y in 0..height {
        for x in 0..width {
            if x == base_x && y == base_y {
                tiles.push(Tile::Base);
                continue;
            }

            let n = perlin.get([x as f64 * scale, y as f64 * scale]);
            if n > 0.35 {
                tiles.push(Tile::Obstacle);
                continue;
            }

            let resource_roll = rng.gen_ratio(1, 25);
            if resource_roll {
                let kind = if rng.gen_bool(0.5) {
                    ResourceType::Energy
                } else {
                    ResourceType::Crystal
                };
                let amount = rng.gen_range(50..=200);
                tiles.push(Tile::Resource { kind, amount });
            } else {
                tiles.push(Tile::Empty);
            }
        }
    }

    Ok(Map {
        width,
        height,
        tiles,
    })
}
