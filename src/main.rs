use my_app::generate_map;

fn main() {
    let map = generate_map(42, 40, 20);
    println!(
        "Map générée {}x{} (sample center tile: {:?})",
        map.width,
        map.height,
        map.get(map.width / 2, map.height / 2)
    );
}
