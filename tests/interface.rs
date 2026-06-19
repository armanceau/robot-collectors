use my_app::{Map, ResourceType, RobotKind, SimState, Tile, generate_map, start_simulation};

#[test]
fn start_simulation_initializes_expected_robot_count() {
    let map = Map {
        width: 7,
        height: 5,
        tiles: vec![Tile::Empty; 35],
    };

    let sim = start_simulation(map);
    let state = sim.read().unwrap();

    assert_eq!(state.robots.len(), 4);
    assert_eq!(
        state
            .robots
            .iter()
            .filter(|robot| robot.kind == RobotKind::Scout)
            .count(),
        2
    );
    assert_eq!(
        state
            .robots
            .iter()
            .filter(|robot| robot.kind == RobotKind::Collector)
            .count(),
        2
    );
}

#[test]
fn start_simulation_keeps_map_dimensions_and_zero_counters() {
    let map = generate_map(123, 9, 6).expect("map should be generated");
    let sim = start_simulation(map);
    let state = sim.read().unwrap();

    assert_eq!(state.map_width, 9);
    assert_eq!(state.map_height, 6);
    assert_eq!(state.total_energy, 0);
    assert_eq!(state.total_crystal, 0);
    assert!(state.known_resources.is_empty());
}

#[test]
fn sim_state_can_represent_resources_and_base_tiles() {
    let map = Map {
        width: 3,
        height: 3,
        tiles: vec![
            Tile::Empty,
            Tile::Resource {
                kind: ResourceType::Energy,
                amount: 100,
            },
            Tile::Base,
            Tile::Resource {
                kind: ResourceType::Crystal,
                amount: 80,
            },
            Tile::Obstacle,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
            Tile::Empty,
        ],
    };

    let state = SimState {
        robots: vec![],
        map_tiles: map.tiles,
        map_width: map.width,
        map_height: map.height,
        total_energy: 0,
        total_crystal: 0,
        known_resources: vec![],
    };

    assert!(matches!(
        state.map_tiles[1],
        Tile::Resource {
            kind: ResourceType::Energy,
            ..
        }
    ));
    assert!(matches!(state.map_tiles[2], Tile::Base));
    assert!(matches!(
        state.map_tiles[3],
        Tile::Resource {
            kind: ResourceType::Crystal,
            ..
        }
    ));
}
