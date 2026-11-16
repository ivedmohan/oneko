//! Port of the oneko.js state machine. Positions are the cat's centre, in screen units.

use crate::sprites::{Dir, Side, Sprite};
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum IdleAnim {
    Sleeping,
    ScratchSelf,
    ScratchWall(Side),
}

pub struct Cat {
    pub x: f64,
    pub y: f64,
    /// Cat size in screen units (32 in the original).
    pub size: f64,
    /// Screen bounds in units, used to decide which wall to scratch.
    pub bounds: (f64, f64),
    frame_count: u64,
    idle_time: u32,
    idle_anim: Option<IdleAnim>,
    idle_frame: u32,
    sprite: (Sprite, usize),
}

impl Cat {
    pub fn new(x: f64, y: f64, size: f64, bounds: (f64, f64)) -> Self {
        Self {
            x,
            y,
            size,
            bounds,
            frame_count: 0,
            idle_time: 0,
            idle_anim: None,
            idle_frame: 0,
            sprite: (Sprite::Idle, 0),
        }
    }

    pub fn sprite(&self) -> (Sprite, usize) {
        self.sprite
    }

    fn speed(&self) -> f64 {
        self.size * 10.0 / 32.0
    }

    fn set_sprite(&mut self, sprite: Sprite, frame: usize) {
        self.sprite = (sprite, frame);
    }

    /// Advance one 100ms tick toward the cursor.
    pub fn tick<R: Rng>(&mut self, mouse: (f64, f64), rng: &mut R) {
        self.frame_count += 1;
        let dx = self.x - mouse.0;
        let dy = self.y - mouse.1;
        let dist = (dx * dx + dy * dy).sqrt();
        let speed = self.speed();

        if dist < speed || dist < self.size * 1.5 {
            self.idle(rng);
            return;
        }

        self.idle_anim = None;
        self.idle_frame = 0;

        if self.idle_time > 1 {
            self.set_sprite(Sprite::Alert, 0);
            self.idle_time = self.idle_time.min(7) - 1;
            return;
        }

        let ny = dy / dist;
        let nx = dx / dist;
        let n = ny > 0.5;
        let s = ny < -0.5;
        let w = nx > 0.5;
        let e = nx < -0.5;
        let dir = match (n, s, w, e) {
            (true, _, true, _) => Dir::NW,
            (true, _, _, true) => Dir::NE,
            (true, _, _, _) => Dir::N,
            (_, true, true, _) => Dir::SW,
            (_, true, _, true) => Dir::SE,
            (_, true, _, _) => Dir::S,
            (_, _, true, _) => Dir::W,
            _ => Dir::E,
        };
        self.set_sprite(Sprite::Walk(dir), self.frame_count as usize);

        self.x -= nx * speed;
        self.y -= ny * speed;
        self.x = self.x.clamp(self.size / 2.0, self.bounds.0 - self.size / 2.0);
        self.y = self.y.clamp(self.size / 2.0, self.bounds.1 - self.size / 2.0);
    }

    fn idle<R: Rng>(&mut self, rng: &mut R) {
        self.idle_time += 1;

        // Roughly every 20 seconds while idle, pick something to do.
        if self.idle_time > 10 && self.idle_anim.is_none() && rng.gen_range(0..200) == 0 {
            let mut options = vec![IdleAnim::Sleeping, IdleAnim::ScratchSelf];
            if self.x < self.size {
                options.push(IdleAnim::ScratchWall(Side::W));
            }
            if self.y < self.size {
                options.push(IdleAnim::ScratchWall(Side::N));
            }
            if self.x > self.bounds.0 - self.size {
                options.push(IdleAnim::ScratchWall(Side::E));
            }
            if self.y > self.bounds.1 - self.size {
                options.push(IdleAnim::ScratchWall(Side::S));
            }
            self.idle_anim = Some(options[rng.gen_range(0..options.len())]);
        }

        match self.idle_anim {
            Some(IdleAnim::Sleeping) => {
                if self.idle_frame < 8 {
                    self.set_sprite(Sprite::Tired, 0);
                } else {
                    self.set_sprite(Sprite::Sleeping, (self.idle_frame / 4) as usize);
                    if self.idle_frame > 192 {
                        self.reset_idle();
                        return;
                    }
                }
            }
            Some(IdleAnim::ScratchSelf) => {
                self.set_sprite(Sprite::ScratchSelf, self.idle_frame as usize);
                if self.idle_frame > 9 {
                    self.reset_idle();
                    return;
                }
            }
            Some(IdleAnim::ScratchWall(side)) => {
                self.set_sprite(Sprite::ScratchWall(side), self.idle_frame as usize);
                if self.idle_frame > 9 {
                    self.reset_idle();
                    return;
                }
            }
            None => {
                self.set_sprite(Sprite::Idle, 0);
                return;
            }
        }
        self.idle_frame += 1;
    }

    fn reset_idle(&mut self) {
        self.idle_anim = None;
        self.idle_frame = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn walks_toward_cursor_and_stops() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut cat = Cat::new(32.0, 32.0, 32.0, (1000.0, 1000.0));
        let target = (500.0, 300.0);
        let start = (cat.x, cat.y);
        for _ in 0..200 {
            cat.tick(target, &mut rng);
        }
        let d = ((cat.x - target.0).powi(2) + (cat.y - target.1).powi(2)).sqrt();
        assert!(d < 48.0, "cat should settle near cursor, got {d}");
        assert_ne!((cat.x, cat.y), start);
        assert_eq!(cat.sprite().0, Sprite::Idle);
    }

    #[test]
    fn alerts_before_moving_after_long_idle() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut cat = Cat::new(100.0, 100.0, 32.0, (1000.0, 1000.0));
        for _ in 0..5 {
            cat.tick((100.0, 100.0), &mut rng);
        }
        cat.tick((600.0, 600.0), &mut rng);
        assert_eq!(cat.sprite().0, Sprite::Alert);
    }
}
