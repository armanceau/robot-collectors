use my_app::generate_map;

fn main() {
    match generate_map(42, 40, 20) {
        Ok(map) => {
            println!(
                "Map générée {}x{} (sample center tile: {:?})",
                map.width,
                map.height,
                map.get(map.width / 2, map.height / 2)
            );
        }
        Err(err) => {
            eprintln!("Erreur de génération de map: {}", err);
        }
    }
}
