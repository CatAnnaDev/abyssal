use crate::map::Map;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
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
        Pathfinder::ALL[i.rem_euclid(Pathfinder::ALL.len() as i32) as usize]
    }
}

struct Scratch {
    len: usize,
    vgen: u32,
    bgen: u32,
    seen: Vec<u32>,
    closed: Vec<u32>,
    g: Vec<i32>,
    came: Vec<i32>,
    block: Vec<u32>,
    heap: BinaryHeap<Reverse<(i32, i32)>>,
    queue: VecDeque<i32>,
}

impl Scratch {
    fn new() -> Self {
        Scratch {
            len: 0,
            vgen: 0,
            bgen: 0,
            seen: Vec::new(),
            closed: Vec::new(),
            g: Vec::new(),
            came: Vec::new(),
            block: Vec::new(),
            heap: BinaryHeap::new(),
            queue: VecDeque::new(),
        }
    }

    fn begin(&mut self, n: usize, blocked: &[(i32, i32)], w: i32) {
        if self.len < n {
            self.seen.resize(n, 0);
            self.closed.resize(n, 0);
            self.g.resize(n, 0);
            self.came.resize(n, 0);
            self.block.resize(n, 0);
            self.len = n;
        }
        self.vgen = self.vgen.wrapping_add(1);
        if self.vgen == 0 {
            for v in self.seen.iter_mut() {
                *v = 0;
            }
            for v in self.closed.iter_mut() {
                *v = 0;
            }
            self.vgen = 1;
        }
        self.bgen = self.bgen.wrapping_add(1);
        if self.bgen == 0 {
            for v in self.block.iter_mut() {
                *v = 0;
            }
            self.bgen = 1;
        }
        for &(bx, by) in blocked {
            if bx >= 0 && by >= 0 {
                let i = (by * w + bx) as usize;
                if i < n {
                    self.block[i] = self.bgen;
                }
            }
        }
        self.heap.clear();
        self.queue.clear();
    }

    #[inline]
    fn is_block(&self, i: usize) -> bool {
        self.block[i] == self.bgen
    }
    #[inline]
    fn gval(&self, i: usize) -> i32 {
        if self.seen[i] == self.vgen {
            self.g[i]
        } else {
            i32::MAX
        }
    }
    #[inline]
    fn relax(&mut self, i: usize, g: i32, parent: i32) {
        self.seen[i] = self.vgen;
        self.g[i] = g;
        self.came[i] = parent;
    }
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch::new());
}

fn reconstruct(came: &[i32], start: i32, goal: i32, w: i32, sx: i32, sy: i32) -> (i32, i32) {
    let mut cur = goal;
    while came[cur as usize] != start && came[cur as usize] != -1 {
        cur = came[cur as usize];
    }
    (cur % w - sx, cur / w - sy)
}

pub fn step_to(pf: Pathfinder, map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)]) -> Option<(i32, i32)> {
    match pf {
        Pathfinder::Bfs => bfs_point(map, sx, sy, gx, gy, blocked).0,
        Pathfinder::AStar => best_first(map, sx, sy, gx, gy, blocked, true, true, false, false, true).0,
        Pathfinder::Greedy => best_first(map, sx, sy, gx, gy, blocked, false, true, false, false, true).0,
        Pathfinder::Dijkstra => best_first(map, sx, sy, gx, gy, blocked, true, false, false, true, false).0,
        Pathfinder::Diagonal => best_first(map, sx, sy, gx, gy, blocked, true, true, true, false, true).0,
    }
}

pub fn path_to(pf: Pathfinder, map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if sx == gx && sy == gy || !map.in_bounds(sx, sy) {
        return Vec::new();
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    let diag = pf == Pathfinder::Diagonal;
    let (use_g, use_h, weighted, hard_block) = match pf {
        Pathfinder::Bfs => (true, false, false, true),
        Pathfinder::AStar => (true, true, false, true),
        Pathfinder::Greedy => (false, true, false, true),
        Pathfinder::Dijkstra => (true, false, true, false),
        Pathfinder::Diagonal => (true, true, false, true),
    };
    let dirs: &[(i32, i32)] = if diag { &STEPS8 } else { &STEPS };
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = (sy * w + sx) as usize;
        s.relax(start, 0, start as i32);
        s.heap.push(Reverse((0, start as i32)));
        let mut goal_i = -1i32;
        while let Some(Reverse((_, ci))) = s.heap.pop() {
            let cu = ci as usize;
            if s.closed[cu] == s.vgen {
                continue;
            }
            s.closed[cu] = s.vgen;
            let cx = ci % w;
            let cy = ci / w;
            if cx == gx && cy == gy {
                goal_i = ci;
                break;
            }
            let cg = s.g[cu];
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
                let ni = (ny * w + nx) as usize;
                let costly = s.is_block(ni);
                if costly && !goal && hard_block {
                    continue;
                }
                let base = if dx != 0 && dy != 0 { 14 } else { 10 };
                let extra = if weighted && costly { 200 } else { 0 };
                let ng = cg.saturating_add(base + extra);
                if ng < s.gval(ni) {
                    s.relax(ni, ng, ci);
                    let hh = if use_h {
                        if diag {
                            (nx - gx).abs().max((ny - gy).abs()) * 10
                        } else {
                            ((nx - gx).abs() + (ny - gy).abs()) * 10
                        }
                    } else {
                        0
                    };
                    let pri = (if use_g { ng } else { 0 }).saturating_add(hh);
                    s.heap.push(Reverse((pri, ni as i32)));
                }
            }
        }
        if goal_i < 0 {
            return Vec::new();
        }
        let mut rev = Vec::new();
        let mut cur = goal_i;
        while cur != start as i32 {
            rev.push((cur % w, cur / w));
            cur = s.came[cur as usize];
        }
        rev.reverse();
        rev
    })
}

pub fn search_cost(pf: Pathfinder, map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)]) -> u32 {
    match pf {
        Pathfinder::Bfs => bfs_point(map, sx, sy, gx, gy, blocked).1,
        Pathfinder::AStar => best_first(map, sx, sy, gx, gy, blocked, true, true, false, false, true).1,
        Pathfinder::Greedy => best_first(map, sx, sy, gx, gy, blocked, false, true, false, false, true).1,
        Pathfinder::Dijkstra => best_first(map, sx, sy, gx, gy, blocked, true, false, false, true, false).1,
        Pathfinder::Diagonal => best_first(map, sx, sy, gx, gy, blocked, true, true, true, false, true).1,
    }
}

fn bfs_point(map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)]) -> (Option<(i32, i32)>, u32) {
    if sx == gx && sy == gy || !map.in_bounds(sx, sy) {
        return (None, 0);
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = sy * w + sx;
        s.seen[start as usize] = s.vgen;
        s.came[start as usize] = start;
        s.queue.push_back(start);
        let mut nodes = 0u32;
        while let Some(ci) = s.queue.pop_front() {
            nodes += 1;
            let cx = ci % w;
            let cy = ci / w;
            if cx == gx && cy == gy {
                return (Some(reconstruct(&s.came, start, ci, w, sx, sy)), nodes);
            }
            for (dx, dy) in STEPS {
                let nx = cx + dx;
                let ny = cy + dy;
                if !map.in_bounds(nx, ny) {
                    continue;
                }
                let ni = (ny * w + nx) as usize;
                if s.seen[ni] == s.vgen {
                    continue;
                }
                let goal = nx == gx && ny == gy;
                if !goal && (!map.is_walkable(nx, ny) || s.is_block(ni)) {
                    continue;
                }
                s.seen[ni] = s.vgen;
                s.came[ni] = ci;
                if goal {
                    return (Some(reconstruct(&s.came, start, ni as i32, w, sx, sy)), nodes);
                }
                s.queue.push_back(ni as i32);
            }
        }
        (None, nodes)
    })
}

#[allow(clippy::too_many_arguments)]
fn best_first(map: &Map, sx: i32, sy: i32, gx: i32, gy: i32, blocked: &[(i32, i32)], use_g: bool, use_h: bool, diag: bool, weighted: bool, hard_block: bool) -> (Option<(i32, i32)>, u32) {
    if sx == gx && sy == gy || !map.in_bounds(sx, sy) {
        return (None, 0);
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    let dirs: &[(i32, i32)] = if diag { &STEPS8 } else { &STEPS };
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = (sy * w + sx) as usize;
        s.relax(start, 0, start as i32);
        s.heap.push(Reverse((0, start as i32)));
        let mut nodes = 0u32;
        while let Some(Reverse((_, ci))) = s.heap.pop() {
            let cu = ci as usize;
            if s.closed[cu] == s.vgen {
                continue;
            }
            s.closed[cu] = s.vgen;
            nodes += 1;
            let cx = ci % w;
            let cy = ci / w;
            if cx == gx && cy == gy {
                return (Some(reconstruct(&s.came, start as i32, ci, w, sx, sy)), nodes);
            }
            let cg = s.g[cu];
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
                let ni = (ny * w + nx) as usize;
                let costly = s.is_block(ni);
                if costly && !goal && hard_block {
                    continue;
                }
                let base = if dx != 0 && dy != 0 { 14 } else { 10 };
                let extra = if weighted && costly { 200 } else { 0 };
                let ng = cg.saturating_add(base + extra);
                if ng < s.gval(ni) {
                    s.relax(ni, ng, ci);
                    let hh = if use_h {
                        if diag {
                            (nx - gx).abs().max((ny - gy).abs()) * 10
                        } else {
                            ((nx - gx).abs() + (ny - gy).abs()) * 10
                        }
                    } else {
                        0
                    };
                    let pri = (if use_g { ng } else { 0 }).saturating_add(hh);
                    s.heap.push(Reverse((pri, ni as i32)));
                }
            }
        }
        (None, nodes)
    })
}

pub fn step_toward(map: &Map, sx: i32, sy: i32, blocked: &[(i32, i32)], is_goal: impl Fn(i32, i32) -> bool) -> Option<(i32, i32)> {
    if is_goal(sx, sy) || !map.in_bounds(sx, sy) {
        return None;
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = sy * w + sx;
        s.seen[start as usize] = s.vgen;
        s.came[start as usize] = start;
        s.queue.push_back(start);
        while let Some(ci) = s.queue.pop_front() {
            let cx = ci % w;
            let cy = ci / w;
            for (dx, dy) in STEPS {
                let nx = cx + dx;
                let ny = cy + dy;
                if !map.in_bounds(nx, ny) {
                    continue;
                }
                let ni = (ny * w + nx) as usize;
                if s.seen[ni] == s.vgen {
                    continue;
                }
                let goal = is_goal(nx, ny);
                if !goal && (!map.is_walkable(nx, ny) || s.is_block(ni)) {
                    continue;
                }
                s.seen[ni] = s.vgen;
                s.came[ni] = ci;
                if goal {
                    return Some(reconstruct(&s.came, start, ni as i32, w, sx, sy));
                }
                s.queue.push_back(ni as i32);
            }
        }
        None
    })
}

pub fn nearest_goal(map: &Map, sx: i32, sy: i32, blocked: &[(i32, i32)], is_goal: impl Fn(i32, i32) -> bool) -> Option<(i32, i32)> {
    if !map.in_bounds(sx, sy) {
        return None;
    }
    let w = map.width;
    let n = (map.width * map.height) as usize;
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = sy * w + sx;
        s.seen[start as usize] = s.vgen;
        s.queue.push_back(start);
        while let Some(ci) = s.queue.pop_front() {
            let cx = ci % w;
            let cy = ci / w;
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
                if s.seen[ni] == s.vgen || !map.is_walkable(nx, ny) || s.is_block(ni) {
                    continue;
                }
                s.seen[ni] = s.vgen;
                s.queue.push_back(ni as i32);
            }
        }
        None
    })
}

pub fn bfs_field(map: &Map, sx: i32, sy: i32, blocked: &[(i32, i32)]) -> Vec<i32> {
    let w = map.width;
    let n = (map.width * map.height) as usize;
    let mut dist = vec![-1i32; n];
    if !map.in_bounds(sx, sy) {
        return dist;
    }
    SCRATCH.with(|s| {
        let s = &mut *s.borrow_mut();
        s.begin(n, blocked, w);
        let start = (sy * w + sx) as usize;
        dist[start] = 0;
        s.queue.push_back(start as i32);
        while let Some(ci) = s.queue.pop_front() {
            let cx = ci % w;
            let cy = ci / w;
            let d = dist[ci as usize];
            for (dx, dy) in STEPS {
                let nx = cx + dx;
                let ny = cy + dy;
                if !map.in_bounds(nx, ny) {
                    continue;
                }
                let ni = (ny * w + nx) as usize;
                if dist[ni] >= 0 || !map.is_walkable(nx, ny) || s.is_block(ni) {
                    continue;
                }
                dist[ni] = d + 1;
                s.queue.push_back(ni as i32);
            }
        }
    });
    dist
}
