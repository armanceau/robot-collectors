use std::{
    collections::VecDeque,
    sync::{Arc, RwLock, mpsc},
    thread,
    time::Duration,
};

use rand::Rng;

use crate::map_generation::{Map, ResourceType, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotKind {
    Scout,
    Collector,
}

#[derive(Debug, Clone)]
pub struct RobotState {
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub kind: RobotKind,
    pub carrying: u64,
    pub carrying_kind: Option<ResourceType>,
}

#[derive(Debug, Clone)]
pub struct SimState {
    pub robots: Vec<RobotState>,
    pub map_tiles: Vec<Tile>,
    pub map_width: usize,
    pub map_height: usize,
    pub total_energy: u64,
    pub total_crystal: u64,
    pub known_resources: Vec<(usize, usize, ResourceType)>,
}

#[derive(Debug)]
enum RobotMessage {
    ResourceDiscovered { x: usize, y: usize, kind: ResourceType },
    ResourceCollected { x: usize, y: usize },
    ResourceUnloaded { kind: ResourceType, amount: u64 },
}

enum CollectorAction {
    Move(usize, usize),
    Collect(usize, usize),
    Unload,
    Idle,
}

/// Starts the simulation from a generated map.
/// Returns a shared, thread-safe state that the UI can read each frame.
pub fn start_simulation(map: Map) -> Arc<RwLock<SimState>> {
    let base_x = map.width / 2;
    let base_y = map.height / 2;

    let mut robots = Vec::new();
    for i in 0..2usize {
        robots.push(RobotState {
            id: i,
            x: base_x,
            y: base_y,
            kind: RobotKind::Scout,
            carrying: 0,
            carrying_kind: None,
        });
    }
    for i in 0..2usize {
        robots.push(RobotState {
            id: 2 + i,
            x: base_x,
            y: base_y,
            kind: RobotKind::Collector,
            carrying: 0,
            carrying_kind: None,
        });
    }

    let state = Arc::new(RwLock::new(SimState {
        robots,
        map_tiles: map.tiles,
        map_width: map.width,
        map_height: map.height,
        total_energy: 0,
        total_crystal: 0,
        known_resources: Vec::new(),
    }));

    let (tx, rx) = mpsc::channel::<RobotMessage>();

    for i in 0..2usize {
        let s = Arc::clone(&state);
        let t = tx.clone();
        thread::spawn(move || run_scout(i, s, t));
    }
    for i in 0..2usize {
        let s = Arc::clone(&state);
        let t = tx.clone();
        thread::spawn(move || run_collector(2 + i, s, t));
    }
    {
        let s = Arc::clone(&state);
        thread::spawn(move || run_coordinator(s, rx));
    }

    state
}

fn run_scout(id: usize, state: Arc<RwLock<SimState>>, tx: mpsc::Sender<RobotMessage>) {
    let mut rng = rand::thread_rng();
    loop {
        let (next, discovered) = {
            let s = state.read().unwrap();
            let r = &s.robots[id];
            let next = random_step(r.x, r.y, s.map_width, s.map_height, &s.map_tiles, &mut rng);
            let discovered = next.and_then(|(nx, ny)| {
                match s.map_tiles[ny * s.map_width + nx] {
                    Tile::Resource { kind, .. } => Some((nx, ny, kind)),
                    _ => None,
                }
            });
            (next, discovered)
        };

        if let Some((nx, ny)) = next {
            let mut s = state.write().unwrap();
            s.robots[id].x = nx;
            s.robots[id].y = ny;
        }
        if let Some((x, y, kind)) = discovered {
            let _ = tx.send(RobotMessage::ResourceDiscovered { x, y, kind });
        }

        thread::sleep(Duration::from_millis(120));
    }
}

fn run_collector(id: usize, state: Arc<RwLock<SimState>>, tx: mpsc::Sender<RobotMessage>) {
    let mut rng = rand::thread_rng();
    loop {
        let action = {
            let s = state.read().unwrap();
            let r = &s.robots[id];
            let base_x = s.map_width / 2;
            let base_y = s.map_height / 2;

            if r.carrying > 0 {
                if r.x == base_x && r.y == base_y {
                    CollectorAction::Unload
                } else {
                    match bfs(&s.map_tiles, s.map_width, s.map_height, (r.x, r.y), (base_x, base_y)) {
                        Some((nx, ny)) => CollectorAction::Move(nx, ny),
                        None => CollectorAction::Idle,
                    }
                }
            } else {
                let closest = s
                    .known_resources
                    .iter()
                    .min_by_key(|(rx, ry, _)| {
                        let dx = (*rx as isize - r.x as isize).unsigned_abs();
                        let dy = (*ry as isize - r.y as isize).unsigned_abs();
                        dx + dy
                    })
                    .copied();

                if let Some((res_x, res_y, _)) = closest {
                    if r.x == res_x && r.y == res_y {
                        CollectorAction::Collect(res_x, res_y)
                    } else {
                        match bfs(&s.map_tiles, s.map_width, s.map_height, (r.x, r.y), (res_x, res_y)) {
                            Some((nx, ny)) => CollectorAction::Move(nx, ny),
                            None => CollectorAction::Idle,
                        }
                    }
                } else {
                    match random_step(r.x, r.y, s.map_width, s.map_height, &s.map_tiles, &mut rng) {
                        Some((nx, ny)) => CollectorAction::Move(nx, ny),
                        None => CollectorAction::Idle,
                    }
                }
            }
        };

        match action {
            CollectorAction::Move(nx, ny) => {
                let mut s = state.write().unwrap();
                s.robots[id].x = nx;
                s.robots[id].y = ny;
            }
            CollectorAction::Collect(res_x, res_y) => {
                let collected = {
                    let mut s = state.write().unwrap();
                    let idx = res_y * s.map_width + res_x;
                    if let Tile::Resource { kind, amount } = s.map_tiles[idx] {
                        s.map_tiles[idx] = if amount <= 1 {
                            Tile::Empty
                        } else {
                            Tile::Resource { kind, amount: amount - 1 }
                        };
                        s.robots[id].carrying += 1;
                        s.robots[id].carrying_kind = Some(kind);
                        true
                    } else {
                        false
                    }
                };
                if collected {
                    let _ = tx.send(RobotMessage::ResourceCollected { x: res_x, y: res_y });
                }
            }
            CollectorAction::Unload => {
                let (kind, amount) = {
                    let mut s = state.write().unwrap();
                    let k = s.robots[id].carrying_kind.take();
                    let a = std::mem::replace(&mut s.robots[id].carrying, 0);
                    (k, a)
                };
                if let Some(k) = kind {
                    let _ = tx.send(RobotMessage::ResourceUnloaded { kind: k, amount });
                }
            }
            CollectorAction::Idle => {}
        }

        thread::sleep(Duration::from_millis(180));
    }
}

fn run_coordinator(state: Arc<RwLock<SimState>>, rx: mpsc::Receiver<RobotMessage>) {
    for msg in rx {
        let mut s = state.write().unwrap();
        match msg {
            RobotMessage::ResourceDiscovered { x, y, kind } => {
                let idx = y * s.map_width + x;
                if matches!(s.map_tiles[idx], Tile::Resource { .. })
                    && !s.known_resources.iter().any(|&(rx, ry, _)| rx == x && ry == y)
                {
                    s.known_resources.push((x, y, kind));
                }
            }
            RobotMessage::ResourceCollected { x, y } => {
                let idx = y * s.map_width + x;
                if s.map_tiles[idx] == Tile::Empty {
                    s.known_resources.retain(|&(rx, ry, _)| !(rx == x && ry == y));
                }
            }
            RobotMessage::ResourceUnloaded { kind, amount } => match kind {
                ResourceType::Energy => s.total_energy += amount,
                ResourceType::Crystal => s.total_crystal += amount,
            },
        }
    }
}

/// BFS pathfinding. Returns the next step to take from `from` towards `to`,
/// or None if already there or no path exists.
fn bfs(
    tiles: &[Tile],
    width: usize,
    height: usize,
    from: (usize, usize),
    to: (usize, usize),
) -> Option<(usize, usize)> {
    if from == to {
        return None;
    }

    let mut visited = vec![false; width * height];
    let mut parent: Vec<Option<usize>> = vec![None; width * height];
    let mut queue = VecDeque::new();

    let start_idx = from.1 * width + from.0;
    visited[start_idx] = true;
    queue.push_back(from);

    let dirs: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    'outer: while let Some((cx, cy)) = queue.pop_front() {
        for &(dx, dy) in &dirs {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            let idx = ny * width + nx;
            if visited[idx] || tiles[idx] == Tile::Obstacle {
                continue;
            }
            visited[idx] = true;
            parent[idx] = Some(cy * width + cx);
            if (nx, ny) == to {
                break 'outer;
            }
            queue.push_back((nx, ny));
        }
    }

    let to_idx = to.1 * width + to.0;
    if !visited[to_idx] {
        return None;
    }

    let mut cur_idx = to_idx;
    loop {
        match parent[cur_idx] {
            Some(prev_idx) if prev_idx == start_idx => {
                return Some((cur_idx % width, cur_idx / width));
            }
            Some(prev_idx) => cur_idx = prev_idx,
            None => return None,
        }
    }
}

fn random_step(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    tiles: &[Tile],
    rng: &mut impl Rng,
) -> Option<(usize, usize)> {
    let dirs: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let valid: Vec<(usize, usize)> = dirs
        .iter()
        .filter_map(|&(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                return None;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if tiles[ny * width + nx] == Tile::Obstacle {
                return None;
            }
            Some((nx, ny))
        })
        .collect();

    if valid.is_empty() {
        None
    } else {
        Some(valid[rng.gen_range(0..valid.len())])
    }
}
