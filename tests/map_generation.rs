use my_app::generate_map;

#[test]
fn same_seed_generates_same_map() {
    let map_a = generate_map(12345, 32, 16);
    let map_b = generate_map(12345, 32, 16);

    assert_eq!(map_a, map_b);
}

#[test]
fn different_seed_generates_different_map() {
    let map_a = generate_map(12345, 32, 16);
    let map_b = generate_map(67890, 32, 16);

    assert_ne!(map_a, map_b);
}
