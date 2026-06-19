use std::{thread, time::Duration};

use my_app::{Map, ResourceType, RobotKind, Tile, generate_map, start_simulation};

#[test]
fn all_robots_start_at_base_position() {
    let map = generate_map(99, 10, 10).unwrap();
    let base_x = map.width / 2;
    let base_y = map.height / 2;
    let sim = start_simulation(map);
    let state = sim.read().unwrap();

    for robot in &state.robots {
        assert_eq!(robot.x, base_x, "robot {} devrait démarrer à base_x={}", robot.id, base_x);
        assert_eq!(robot.y, base_y, "robot {} devrait démarrer à base_y={}", robot.id, base_y);
    }
}

#[test]
fn simulation_has_two_scouts_and_two_collectors() {
    let map = generate_map(100, 8, 8).unwrap();
    let sim = start_simulation(map);
    let state = sim.read().unwrap();

    let scouts = state.robots.iter().filter(|r| r.kind == RobotKind::Scout).count();
    let collectors = state.robots.iter().filter(|r| r.kind == RobotKind::Collector).count();

    assert_eq!(scouts, 2, "il doit y avoir exactement 2 éclaireurs");
    assert_eq!(collectors, 2, "il doit y avoir exactement 2 collecteurs");
}

#[test]
fn simulation_starts_with_zero_collected_resources() {
    let map = generate_map(42, 10, 10).unwrap();
    let sim = start_simulation(map);
    let state = sim.read().unwrap();

    assert_eq!(state.total_energy, 0);
    assert_eq!(state.total_crystal, 0);
    assert!(state.known_resources.is_empty());
}

#[test]
fn scouts_discover_resources_near_base() {
    // Carte 5x5 avec une ressource adjacente à la base — les éclaireurs doivent la trouver rapidement
    let width = 5;
    let height = 5;
    let base_x = width / 2;
    let base_y = height / 2;

    let mut tiles = vec![Tile::Empty; width * height];
    tiles[base_y * width + base_x] = Tile::Base;
    tiles[base_y * width + (base_x + 1)] = Tile::Resource {
        kind: ResourceType::Energy,
        amount: 100,
    };

    let sim = start_simulation(Map { width, height, tiles });

    thread::sleep(Duration::from_millis(2000));

    let state = sim.read().unwrap();
    assert!(
        !state.known_resources.is_empty(),
        "les éclaireurs auraient dû découvrir la ressource adjacente en 2 secondes"
    );
}

#[test]
fn collectors_increase_totals_after_collecting() {
    // Carte 5x5 avec une ressource accessible à 1 pas de la base
    let width = 5;
    let height = 5;
    let base_x = width / 2;
    let base_y = height / 2;

    let mut tiles = vec![Tile::Empty; width * height];
    tiles[base_y * width + base_x] = Tile::Base;
    tiles[base_y * width + (base_x + 1)] = Tile::Resource {
        kind: ResourceType::Energy,
        amount: 50,
    };

    let sim = start_simulation(Map { width, height, tiles });

    thread::sleep(Duration::from_millis(5000));

    let state = sim.read().unwrap();
    let total = state.total_energy + state.total_crystal;
    assert!(
        total > 0,
        "les collecteurs auraient dû rapporter des ressources en 5 secondes (total={total})"
    );
}

#[test]
fn depleted_resource_is_removed_from_known_resources() {
    // Ressource avec 1 seule unité — après collecte elle doit disparaître de known_resources
    let width = 5;
    let height = 5;
    let base_x = width / 2;
    let base_y = height / 2;

    let mut tiles = vec![Tile::Empty; width * height];
    tiles[base_y * width + base_x] = Tile::Base;
    tiles[base_y * width + (base_x + 1)] = Tile::Resource {
        kind: ResourceType::Crystal,
        amount: 1,
    };

    let sim = start_simulation(Map { width, height, tiles });

    // Laisser le temps de collecter et retirer la ressource
    thread::sleep(Duration::from_millis(5000));

    let state = sim.read().unwrap();
    let tile_idx = base_y * width + (base_x + 1);
    assert_eq!(
        state.map_tiles[tile_idx],
        Tile::Empty,
        "la ressource épuisée doit devenir Tile::Empty"
    );
    assert!(
        !state.known_resources.iter().any(|&(x, y, _)| x == base_x + 1 && y == base_y),
        "la ressource épuisée doit être retirée de known_resources"
    );
}
