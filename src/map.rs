use crate::rng::Rng;
use serde::{Deserialize, Serialize};

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
        self.x <= other.x + other.w
            && self.x + self.w >= other.x
            && self.y <= other.y + other.h
            && self.y + self.h >= other.y
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
    walkable_total: i32,
    explored_walkable: i32,
}

impl Map {
    pub fn generate(width: i32, height: i32, rng: &mut Rng) -> Self {
        let count = (width * height) as usize;
        let mut map = Map {
            width,
            height,
            tiles: vec![Tile::Wall; count],
            visible: vec![false; count],
            explored: vec![false; count],
            rooms: Vec::new(),
            stairs: (0, 0),
            walkable_total: 0,
            explored_walkable: 0,
        };

        let attempts = 60 + (width * height / 220);
        for _ in 0..attempts {
            let w = rng.between(5, 12);
            let h = rng.between(4, 8);
            let x = rng.between(1, width - w - 1);
            let y = rng.between(1, height - h - 1);
            if w < 3 || h < 3 || x < 1 || y < 1 {
                continue;
            }
            let room = Rect { x, y, w, h };
            if map.rooms.iter().any(|r| room.intersects(r)) {
                continue;
            }
            map.carve_room(&room);
            if let Some(prev) = map.rooms.last().copied() {
                let (px, py) = prev.center();
                let (cx, cy) = room.center();
                if rng.chance(0.5) {
                    map.carve_h(px, cx, py);
                    map.carve_v(py, cy, cx);
                } else {
                    map.carve_v(py, cy, px);
                    map.carve_h(px, cx, cy);
                }
            }
            map.rooms.push(room);
        }

        if let Some(last) = map.rooms.last().copied() {
            let (sx, sy) = last.center();
            map.stairs = (sx, sy);
            let idx = map.idx(sx, sy);
            map.tiles[idx] = Tile::StairsDown;
        }
        map.walkable_total = map.tiles.iter().filter(|&&t| t != Tile::Wall).count() as i32;
        map
    }

    pub fn discovery_percent(&self) -> i32 {
        if self.walkable_total == 0 {
            return 100;
        }
        (self.explored_walkable * 100 / self.walkable_total).min(100)
    }

    pub fn spawn_point(&self) -> (i32, i32) {
        self.rooms.first().map(|r| r.center()).unwrap_or((1, 1))
    }

    fn carve_room(&mut self, room: &Rect) {
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                let i = self.idx(x, y);
                self.tiles[i] = Tile::Floor;
            }
        }
    }

    fn carve_h(&mut self, x1: i32, x2: i32, y: i32) {
        for x in x1.min(x2)..=x1.max(x2) {
            let i = self.idx(x, y);
            if self.tiles[i] == Tile::Wall {
                self.tiles[i] = Tile::Floor;
            }
        }
    }

    fn carve_v(&mut self, y1: i32, y2: i32, x: i32) {
        for y in y1.min(y2)..=y1.max(y2) {
            let i = self.idx(x, y);
            if self.tiles[i] == Tile::Wall {
                self.tiles[i] = Tile::Floor;
            }
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
