use my_app::{MapGenerationError, generate_map};

#[test]
fn same_seed_generates_same_map() {
    let map_a = generate_map(12345, 32, 16).expect("valid dimensions should generate a map");
    let map_b = generate_map(12345, 32, 16).expect("valid dimensions should generate a map");

    assert_eq!(map_a, map_b);
}

#[test]
fn different_seed_generates_different_map() {
    let map_a = generate_map(12345, 32, 16).expect("valid dimensions should generate a map");
    let map_b = generate_map(67890, 32, 16).expect("valid dimensions should generate a map");

    assert_ne!(map_a, map_b);
}

#[test]
fn zero_dimension_is_rejected() {
    let err = generate_map(42, 0, 20).expect_err("zero width should be invalid");

    assert_eq!(
        err,
        MapGenerationError::InvalidDimensions {
            width: 0,
            height: 20
        }
    );
}
