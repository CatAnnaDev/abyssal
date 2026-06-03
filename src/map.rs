use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    StairsDown,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    fn intersects(&self, other: &Rect) -> bool {
        self.x - 1 <= other.x + other.w
            && self.x + self.w + 1 >= other.x
            && self.y - 1 <= other.y + other.h
            && self.y + self.h + 1 >= other.y
    }
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    tiles: Vec<Tile>,
    visible: Vec<bool>,
    explored: Vec<bool>,
    pub rooms: Vec<Rect>,
    pub stairs: (i32, i32),
    spawn: (i32, i32),
    walkable_total: i32,
    explored_walkable: i32,
    #[serde(default)]
    decor: Vec<u8>,
}

impl Map {
    pub fn generate(width: i32, height: i32, rng: &mut Rng) -> Self {
        Map::generate_styled(width, height, 0, rng)
    }

    pub fn generate_styled(width: i32, height: i32, style: i32, rng: &mut Rng) -> Self {
        let count = (width * height) as usize;
        let mut map = Map {
            width,
            height,
            tiles: vec![Tile::Wall; count],
            visible: vec![false; count],
            explored: vec![false; count],
            rooms: Vec::new(),
            stairs: (0, 0),
            spawn: (1, 1),
            walkable_total: 0,
            explored_walkable: 0,
            decor: vec![0u8; count],
        };

        match style {
            0 => map.gen_caves(rng),
            1 => map.gen_rooms(rng, 18, 4, 8, 3, 6, 0.15, false),
            2 => map.gen_rooms(rng, 7, 9, 18, 6, 11, 0.45, true),
            3 => {
                map.gen_rooms(rng, 11, 6, 11, 4, 8, 0.3, false);
                map.scatter_caves(rng);
            }
            _ => map.gen_rooms(rng, 16, 5, 9, 4, 7, 0.25, false),
        }

        map.keep_largest_component();
        if !map.rooms.is_empty() {
            map.add_loops(rng, if style == 4 { 5 } else { 2 });
            map.add_pillars(rng);
        }
        map.fallback_if_tiny(rng);
        map.place_spawn_and_stairs();
        map.scatter_decor(style, rng);
        map.walkable_total = map.tiles.iter().filter(|&&t| t != Tile::Wall).count() as i32;
        map
    }

    fn gen_rooms(&mut self, rng: &mut Rng, attempts: i32, min_w: i32, max_w: i32, min_h: i32, max_h: i32, round_chance: f32, wide: bool) {
        let attempts = attempts + self.width * self.height / 320;
        for _ in 0..attempts {
            let w = rng.between(min_w, max_w + 1);
            let h = rng.between(min_h, max_h + 1);
            if w < 3 || h < 3 || self.width - w - 2 < 2 || self.height - h - 2 < 2 {
                continue;
            }
            let x = rng.between(1, self.width - w - 1);
            let y = rng.between(1, self.height - h - 1);
            let room = Rect { x, y, w, h };
            if self.rooms.iter().any(|r| room.intersects(r)) {
                continue;
            }
            if round_chance > 0.0 && w >= 5 && h >= 5 && rng.chance(round_chance) {
                self.carve_ellipse(&room);
            } else {
                self.carve_room(&room);
            }
            if let Some(prev) = self.rooms.last().copied() {
                let (px, py) = prev.center();
                let (cx, cy) = room.center();
                self.tunnel(px, py, cx, cy, rng, wide);
            }
            self.rooms.push(room);
        }
    }

    fn scatter_caves(&mut self, rng: &mut Rng) {
        let blobs = 2 + rng.below(3) as i32;
        for _ in 0..blobs {
            let mut x = rng.between(4, self.width - 4);
            let mut y = rng.between(4, self.height - 4);
            let steps = 60 + rng.below(80) as i32;
            for _ in 0..steps {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx > 0 && ny > 0 && nx < self.width - 1 && ny < self.height - 1 {
                            let i = self.idx(nx, ny);
                            self.tiles[i] = Tile::Floor;
                        }
                    }
                }
                match rng.below(4) {
                    0 => x += 1,
                    1 => x -= 1,
                    2 => y += 1,
                    _ => y -= 1,
                }
                x = x.clamp(2, self.width - 3);
                y = y.clamp(2, self.height - 3);
            }
        }
    }

    fn gen_caves(&mut self, rng: &mut Rng) {
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if rng.chance(0.46) {
                    let i = self.idx(x, y);
                    self.tiles[i] = Tile::Floor;
                }
            }
        }
        for _ in 0..5 {
            let mut next = self.tiles.clone();
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let walls = self.wall_neighbors(x, y);
                    let i = self.idx(x, y);
                    next[i] = if walls >= 5 { Tile::Wall } else { Tile::Floor };
                }
            }
            self.tiles = next;
        }
        for x in 0..self.width {
            let top = self.idx(x, 0);
            let bot = self.idx(x, self.height - 1);
            self.tiles[top] = Tile::Wall;
            self.tiles[bot] = Tile::Wall;
        }
        for y in 0..self.height {
            let l = self.idx(0, y);
            let r = self.idx(self.width - 1, y);
            self.tiles[l] = Tile::Wall;
            self.tiles[r] = Tile::Wall;
        }
    }

    fn wall_neighbors(&self, x: i32, y: i32) -> i32 {
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if !self.in_bounds(nx, ny) || self.tiles[self.idx(nx, ny)] == Tile::Wall {
                    n += 1;
                }
            }
        }
        n
    }

    fn carve_ellipse(&mut self, room: &Rect) {
        let (cx, cy) = room.center();
        let rx = room.w as f32 / 2.0;
        let ry = room.h as f32 / 2.0;
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                let nx = (x - cx) as f32 / rx;
                let ny = (y - cy) as f32 / ry;
                if nx * nx + ny * ny <= 1.05 {
                    let i = self.idx(x, y);
                    self.tiles[i] = Tile::Floor;
                }
            }
        }
    }

    fn add_loops(&mut self, rng: &mut Rng, extra: i32) {
        if self.rooms.len() < 3 {
            return;
        }
        for _ in 0..extra {
            let a = rng.below(self.rooms.len());
            let mut b = rng.below(self.rooms.len());
            if a == b {
                b = (b + 1) % self.rooms.len();
            }
            let (ax, ay) = self.rooms[a].center();
            let (bx, by) = self.rooms[b].center();
            self.tunnel(ax, ay, bx, by, rng, false);
        }
    }

    fn add_pillars(&mut self, rng: &mut Rng) {
        let rooms = self.rooms.clone();
        for r in rooms {
            if r.w < 7 || r.h < 6 {
                continue;
            }
            let pillars = 1 + rng.below(3) as i32;
            for _ in 0..pillars {
                let px = rng.between(r.x + 2, r.x + r.w - 2);
                let py = rng.between(r.y + 2, r.y + r.h - 2);
                let i = self.idx(px, py);
                self.tiles[i] = Tile::Wall;
            }
        }
    }

    fn keep_largest_component(&mut self) {
        let count = (self.width * self.height) as usize;
        let mut comp = vec![-1i32; count];
        let mut sizes: Vec<i32> = Vec::new();
        for sy in 0..self.height {
            for sx in 0..self.width {
                let si = self.idx(sx, sy);
                if self.tiles[si] == Tile::Wall || comp[si] >= 0 {
                    continue;
                }
                let id = sizes.len() as i32;
                let mut size = 0;
                let mut q = VecDeque::new();
                q.push_back((sx, sy));
                comp[si] = id;
                while let Some((x, y)) = q.pop_front() {
                    size += 1;
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (x + dx, y + dy);
                        if self.in_bounds(nx, ny) {
                            let ni = self.idx(nx, ny);
                            if self.tiles[ni] != Tile::Wall && comp[ni] < 0 {
                                comp[ni] = id;
                                q.push_back((nx, ny));
                            }
                        }
                    }
                }
                sizes.push(size);
            }
        }
        if sizes.is_empty() {
            return;
        }
        let best = sizes.iter().enumerate().max_by_key(|(_, s)| **s).map(|(i, _)| i as i32).unwrap_or(0);
        for i in 0..count {
            if self.tiles[i] != Tile::Wall && comp[i] != best {
                self.tiles[i] = Tile::Wall;
            }
        }
        let width = self.width;
        let height = self.height;
        self.rooms.retain(|r| {
            let (cx, cy) = r.center();
            cx >= 0 && cy >= 0 && cx < width && cy < height && comp[(cy * width + cx) as usize] == best
        });
    }

    fn fallback_if_tiny(&mut self, rng: &mut Rng) {
        let floor = self.tiles.iter().filter(|&&t| t != Tile::Wall).count() as i32;
        if floor >= (self.width * self.height) / 16 {
            return;
        }
        self.tiles = vec![Tile::Wall; (self.width * self.height) as usize];
        self.rooms.clear();
        self.gen_rooms(rng, 16, 5, 9, 4, 7, 0.0, false);
        self.keep_largest_component();
    }

    fn place_spawn_and_stairs(&mut self) {
        let first = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .find(|&(x, y)| self.tiles[self.idx(x, y)] != Tile::Wall);
        let Some(seed) = first else {
            self.spawn = (1, 1);
            self.stairs = (1, 1);
            return;
        };
        let far_from_seed = self.bfs_farthest(seed.0, seed.1);
        self.spawn = far_from_seed;
        let stairs = self.bfs_farthest(self.spawn.0, self.spawn.1);
        self.stairs = stairs;
        let i = self.idx(stairs.0, stairs.1);
        self.tiles[i] = Tile::StairsDown;
        if self.rooms.is_empty() {
            self.rooms.push(Rect { x: self.spawn.0, y: self.spawn.1, w: 1, h: 1 });
        }
    }

    fn bfs_farthest(&self, sx: i32, sy: i32) -> (i32, i32) {
        let count = (self.width * self.height) as usize;
        let mut dist = vec![-1i32; count];
        let mut q = VecDeque::new();
        let si = self.idx(sx, sy);
        dist[si] = 0;
        q.push_back((sx, sy));
        let mut best = (sx, sy);
        let mut best_d = 0;
        while let Some((x, y)) = q.pop_front() {
            let d = dist[self.idx(x, y)];
            if d > best_d {
                best_d = d;
                best = (x, y);
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if self.in_bounds(nx, ny) {
                    let ni = self.idx(nx, ny);
                    if self.tiles[ni] != Tile::Wall && dist[ni] < 0 {
                        dist[ni] = d + 1;
                        q.push_back((nx, ny));
                    }
                }
            }
        }
        best
    }

    fn scatter_decor(&mut self, style: i32, rng: &mut Rng) {
        let palette: &[u8] = match style {
            0 => &[1, 1, 2],
            1 => &[3, 4, 4],
            2 => &[5, 6, 2],
            3 => &[7, 7, 4],
            _ => &[8, 4, 3],
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let i = self.idx(x, y);
                if self.tiles[i] == Tile::Floor && rng.chance(0.14) {
                    self.decor[i] = palette[rng.below(palette.len())];
                }
            }
        }
    }

    pub fn decor_at(&self, x: i32, y: i32) -> u8 {
        if !self.in_bounds(x, y) {
            return 0;
        }
        let i = self.idx(x, y);
        if i < self.decor.len() {
            self.decor[i]
        } else {
            0
        }
    }

    pub fn discovery_percent(&self) -> i32 {
        if self.walkable_total == 0 {
            return 100;
        }
        (self.explored_walkable * 100 / self.walkable_total).min(100)
    }

    pub fn spawn_point(&self) -> (i32, i32) {
        self.spawn
    }

    fn carve_room(&mut self, room: &Rect) {
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                if x <= 0 || y <= 0 || x >= self.width - 1 || y >= self.height - 1 {
                    continue;
                }
                let i = self.idx(x, y);
                self.tiles[i] = Tile::Floor;
            }
        }
    }

    fn tunnel(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, rng: &mut Rng, wide: bool) {
        if rng.chance(0.5) {
            self.carve_h(x1, x2, y1, wide);
            self.carve_v(y1, y2, x2, wide);
        } else {
            self.carve_v(y1, y2, x1, wide);
            self.carve_h(x1, x2, y2, wide);
        }
    }

    fn carve_h(&mut self, x1: i32, x2: i32, y: i32, wide: bool) {
        for x in x1.min(x2)..=x1.max(x2) {
            self.dig(x, y);
            if wide {
                self.dig(x, y + 1);
            }
        }
    }

    fn carve_v(&mut self, y1: i32, y2: i32, x: i32, wide: bool) {
        for y in y1.min(y2)..=y1.max(y2) {
            self.dig(x, y);
            if wide {
                self.dig(x + 1, y);
            }
        }
    }

    fn dig(&mut self, x: i32, y: i32) {
        if x <= 0 || y <= 0 || x >= self.width - 1 || y >= self.height - 1 {
            return;
        }
        let i = self.idx(x, y);
        if self.tiles[i] == Tile::Wall {
            self.tiles[i] = Tile::Floor;
        }
    }

    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        self.tiles[self.idx(x, y)]
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.tiles[self.idx(x, y)] != Tile::Wall
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.visible[self.idx(x, y)]
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.explored[self.idx(x, y)]
    }

    pub fn has_unexplored_neighbor(&self, x: i32, y: i32) -> bool {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if self.in_bounds(nx, ny) {
                let i = self.idx(nx, ny);
                if !self.explored[i] && self.tiles[i] != Tile::Wall {
                    return true;
                }
            }
        }
        false
    }

    pub fn reveal_all(&mut self) {
        for i in 0..self.tiles.len() {
            if !self.explored[i] {
                self.explored[i] = true;
                if self.tiles[i] != Tile::Wall {
                    self.explored_walkable += 1;
                }
            }
            self.visible[i] = true;
        }
    }

    pub fn compute_fov(&mut self, ox: i32, oy: i32, radius: i32) {
        for v in self.visible.iter_mut() {
            *v = false;
        }
        let r2 = radius * radius;
        for ty in (oy - radius).max(0)..=(oy + radius).min(self.height - 1) {
            for tx in (ox - radius).max(0)..=(ox + radius).min(self.width - 1) {
                let dx = tx - ox;
                let dy = ty - oy;
                if dx * dx + dy * dy > r2 {
                    continue;
                }
                if self.line_of_sight(ox, oy, tx, ty) {
                    let i = self.idx(tx, ty);
                    self.visible[i] = true;
                    if !self.explored[i] {
                        self.explored[i] = true;
                        if self.tiles[i] != Tile::Wall {
                            self.explored_walkable += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn line_of_sight(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            if x == x1 && y == y1 {
                return true;
            }
            if !(x == x0 && y == y0) && self.tiles[self.idx(x, y)] == Tile::Wall {
                return false;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}
