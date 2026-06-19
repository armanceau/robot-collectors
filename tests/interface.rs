#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn collect_if_on_resource_updates_counters_and_resource_amount() {
        let mut map = Map {
            width: 1,
            height: 1,
            tiles: vec![Tile::Resource {
                kind: ResourceType::Energy,
                amount: 16,
            }],
        };
        let mut robot = Robot {
            kind: RobotKind::Collector,
            x: 0,
            y: 0,
        };
        let mut energy = 0;
        let mut crystals = 0;

        collect_if_on_resource(&mut map, &mut robot, &mut energy, &mut crystals);

        assert_eq!(energy, 2);
        assert_eq!(crystals, 0);
        assert_eq!(
            map.get(0, 0),
            Tile::Resource {
                kind: ResourceType::Energy,
                amount: 14,
            }
        );
    }

    #[test]
    fn map_to_lines_prefers_robot_icon_over_tile_symbol() {
        let map = Map {
            width: 1,
            height: 1,
            tiles: vec![Tile::Resource {
                kind: ResourceType::Crystal,
                amount: 50,
            }],
        };
        let robots = [Robot {
            kind: RobotKind::Collector,
            x: 0,
            y: 0,
        }];

        let lines = map_to_lines(
            &map,
            &robots,
            Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        );

        assert_eq!(line_text(&lines[0]), "o");
    }

    #[test]
    fn build_stats_panel_contains_expected_interface_labels() {
        let state = SimulationState {
            robots: vec![
                Robot {
                    kind: RobotKind::Scout,
                    x: 0,
                    y: 0,
                },
                Robot {
                    kind: RobotKind::Collector,
                    x: 1,
                    y: 0,
                },
            ],
            collected_energy: 12,
            collected_crystals: 7,
            tick: 5,
        };

        let lines = build_stats_panel(&state);
        let rendered = lines
            .iter()
            .map(|line| line_text(line))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Énergie récoltée : 12"));
        assert!(rendered.contains("Cristaux récoltés : 7"));
        assert!(rendered.contains("Éclaireurs : 1"));
        assert!(rendered.contains("Collecteurs : 1"));
        assert!(rendered.contains("Tour : 5"));
        assert!(rendered.contains("Légende : E énergie | C cristaux | # base"));
    }
}
