use crate::map::Map;
use std::collections::VecDeque;

const STEPS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

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
