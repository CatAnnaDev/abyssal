use crate::map::Map;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

const STEPS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
const STEPS8: [(i32, i32); 8] = [(0, -1), (0, 1), (-1, 0), (1, 0), (-1, -1), (1, -1), (-1, 1), (1, 1)];

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Pathfinder {
    #[default]
    Bfs,
    AStar,
    Dijkstra,
    Greedy,
    Diagonal,
}

impl Pathfinder {
    pub const ALL: [Pathfinder; 5] = [Pathfinder::Bfs, Pathfinder::AStar, Pathfinder::Dijkstra, Pathfinder::Greedy, Pathfinder::Diagonal];

    pub fn label(self) -> &'static str {
        match self {
            Pathfinder::Bfs => "BFS",
            Pathfinder::AStar => "A*",
            Pathfinder::Dijkstra => "Dijkstra",
            Pathfinder::Greedy => "Greedy",
            Pathfinder::Diagonal => "Diagonale",
        }
    }

    pub fn from_index(i: i32) -> Pathfinder {
        Pathfinder::ALL[(i.rem_euclid(Pathfinder::ALL.len() as i32)) as usize]
    }
}

pub fn step_to(pf: Pathfinder, map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)]) -> Option<(i32, i32)> {
    match pf {
        Pathfinder::Bfs => step_toward(map, sx, sy, blocked, |x, y| x == gx && y == gy),
        Pathfinder::AStar => best_first(map, sx, sy, gx, gy, blocked, true, true, false, false, true),
        Pathfinder::Greedy => best_first(map, sx, sy, gx, gy, blocked, false, true, false, false, true),
        Pathfinder::Dijkstra => best_first(map, sx, sy, gx, gy, blocked, true, false, false, true, false),
        Pathfinder::Diagonal => best_first(map, sx, sy, gx, gy, blocked, true, true, true, false, true),
    }
}

#[allow(clippy::too_many_arguments)]
fn best_first(map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, costly: &[(i32, i32)], use_g: bool, use_h: bool, diag: bool, weighted: bool, hard_block: bool) -> Option<(i32, i32)> {
    if sx == gx && sy == gy {
        return None;
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    let mut g = vec![i32::MAX; n];
    let mut came = vec![-1i32; n];
    let start = (sy * w + sx) as usize;
    g[start] = 0;
    let mut heap: BinaryHeap<Reverse<(i32, i32)>> = BinaryHeap::new();
    heap.push(Reverse((0, start as i32)));
    let dirs: &[(i32, i32)] = if diag { &STEPS8 } else { &STEPS };
    let h = |x: i32, y: i32| -> i32 {
        if diag {
            (x - gx).abs().max((y - gy).abs()) * 10
        } else {
            ((x - gx).abs() + (y - gy).abs()) * 10
        }
    };
    while let Some(Reverse((_, ci))) = heap.pop() {
        let cx = ci % w;
        let cy = ci / w;
        if cx == gx && cy == gy {
            return Some(reconstruct(&came, start as i32, ci, w, sx, sy));
        }
        for &(dx, dy) in dirs {
            let nx = cx + dx;
            let ny = cy + dy;
            if !map.in_bounds(nx, ny) {
                continue;
            }
            let goal = nx == gx && ny == gy;
            if !goal && !map.is_walkable(nx, ny) {
                continue;
            }
            let is_costly = costly.iter().any(|&(bx, by)| bx == nx && by == ny);
            if is_costly && !goal {
                if hard_block {
                    continue;
                }
            }
            let base = if dx != 0 && dy != 0 { 14 } else { 10 };
            let extra = if weighted && is_costly { 200 } else { 0 };
            let ni = (ny * w + nx) as usize;
            let ng = g[ci as usize].saturating_add(base + extra);
            if ng < g[ni] {
                g[ni] = ng;
                came[ni] = ci;
                let pri = (if use_g { ng } else { 0 }).saturating_add(if use_h { h(nx, ny) } else { 0 });
                heap.push(Reverse((pri, ni as i32)));
            }
        }
    }
    None
}

fn reconstruct(came: &[i32], start: i32, goal: i32, w: i32, sx: i32, sy: i32) -> (i32, i32) {
    let mut cur = goal;
    while came[cur as usize] != start && came[cur as usize] != -1 {
        cur = came[cur as usize];
    }
    (cur % w - sx, cur / w - sy)
}

pub fn step_toward(
    map: &Map,
    sx: i32,
    sy: i32,
    blocked: &[(i32, i32)],
    is_goal: impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    if is_goal(sx, sy) {
        return None;
    }
    let w = map.width;
    let mut came_from = vec![-1i32; (map.width * map.height) as usize];
    let start = sy * w + sx;
    came_from[start as usize] = start;
    let mut queue = VecDeque::new();
    queue.push_back((sx, sy));

    while let Some((cx, cy)) = queue.pop_front() {
        for (dx, dy) in STEPS {
            let nx = cx + dx;
            let ny = cy + dy;
            if !map.in_bounds(nx, ny) {
                continue;
            }
            let ni = (ny * w + nx) as usize;
            if came_from[ni] != -1 {
                continue;
            }
            let goal = is_goal(nx, ny);
            if !goal {
                if !map.is_walkable(nx, ny) {
                    continue;
                }
                if blocked.iter().any(|&(bx, by)| bx == nx && by == ny) {
                    continue;
                }
            }
            came_from[ni] = cy * w + cx;
            if goal {
                return Some(trace_first_step(&came_from, start, ni as i32, w, sx, sy));
            }
            queue.push_back((nx, ny));
        }
    }
    None
}

pub fn nearest_goal(
    map: &Map,
    sx: i32,
    sy: i32,
    blocked: &[(i32, i32)],
    is_goal: impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    let w = map.width;
    let mut seen = vec![false; (map.width * map.height) as usize];
    seen[(sy * w + sx) as usize] = true;
    let mut queue = VecDeque::new();
    queue.push_back((sx, sy));
    while let Some((cx, cy)) = queue.pop_front() {
        if (cx != sx || cy != sy) && is_goal(cx, cy) {
            return Some((cx, cy));
        }
        for (dx, dy) in STEPS {
            let nx = cx + dx;
            let ny = cy + dy;
            if !map.in_bounds(nx, ny) {
                continue;
            }
            let ni = (ny * w + nx) as usize;
            if seen[ni] || !map.is_walkable(nx, ny) {
                continue;
            }
            if blocked.iter().any(|&(bx, by)| bx == nx && by == ny) {
                continue;
            }
            seen[ni] = true;
            queue.push_back((nx, ny));
        }
    }
    None
}

pub fn bfs_field(map: &Map, sx: i32, sy: i32, blocked: &[(i32, i32)]) -> Vec<i32> {
    let w = map.width;
    let mut dist = vec![-1i32; (map.width * map.height) as usize];
    if !map.in_bounds(sx, sy) {
        return dist;
    }
    dist[(sy * w + sx) as usize] = 0;
    let mut queue = VecDeque::new();
    queue.push_back((sx, sy));
    while let Some((cx, cy)) = queue.pop_front() {
        let d = dist[(cy * w + cx) as usize];
        for (dx, dy) in STEPS {
            let nx = cx + dx;
            let ny = cy + dy;
            if !map.in_bounds(nx, ny) {
                continue;
            }
            let ni = (ny * w + nx) as usize;
            if dist[ni] >= 0 || !map.is_walkable(nx, ny) {
                continue;
            }
            if blocked.iter().any(|&(bx, by)| bx == nx && by == ny) {
                continue;
            }
            dist[ni] = d + 1;
            queue.push_back((nx, ny));
        }
    }
    dist
}

fn trace_first_step(came_from: &[i32], start: i32, goal: i32, w: i32, sx: i32, sy: i32) -> (i32, i32) {
    let mut cur = goal;
    while came_from[cur as usize] != start {
        cur = came_from[cur as usize];
    }
    let nx = cur % w;
    let ny = cur / w;
    (nx - sx, ny - sy)
}
