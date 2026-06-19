pub mod map_generation;
pub mod simulation;

pub use map_generation::{Map, MapGenerationError, ResourceType, Tile, generate_map};
pub use simulation::{RobotKind, RobotState, SimState, start_simulation};
