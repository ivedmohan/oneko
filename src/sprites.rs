//! Sprite sheet: 8 columns x 4 rows of 32px cells, row-major.

use image::{imageops::FilterType, ImageFormat, RgbaImage};

pub const CELL: u32 = 32;

/// The characters shipped with the original X11 oneko, converted by `scripts/xbm2sheet.py`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Character {
    Neko,
    Tora,
    Dog,
    Sakura,
    Tomoyo,
}

impl Character {
    pub const ALL: [Character; 5] = [
        Character::Neko,
        Character::Tora,
        Character::Dog,
        Character::Sakura,
        Character::Tomoyo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Character::Neko => "neko",
            Character::Tora => "tora",
            Character::Dog => "dog",
            Character::Sakura => "sakura",
            Character::Tomoyo => "tomoyo",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Character::Neko => "Neko (white cat)",
            Character::Tora => "Tora (tiger cat)",
            Character::Dog => "Dog",
            Character::Sakura => "Sakura",
            Character::Tomoyo => "Tomoyo",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }

    fn sheet(self) -> &'static [u8] {
        match self {
            Character::Neko => include_bytes!("../assets/neko.png"),
            Character::Tora => include_bytes!("../assets/tora.png"),
            Character::Dog => include_bytes!("../assets/dog.png"),
            Character::Sakura => include_bytes!("../assets/sakura.png"),
            Character::Tomoyo => include_bytes!("../assets/tomoyo.png"),
        }
    }
}
const COLS: u32 = 8;
const ROWS: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    N,
    S,
    E,
    W,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sprite {
    Idle,
    Alert,
    Tired,
    Sleeping,
    ScratchSelf,
    ScratchWall(Side),
    Walk(Dir),
}

impl Sprite {
    /// (column, row) cells in the sheet, in animation order.
    pub fn cells(self) -> &'static [(u32, u32)] {
        match self {
            Sprite::Idle => &[(3, 3)],
            Sprite::Alert => &[(7, 3)],
            Sprite::Tired => &[(3, 2)],
            Sprite::Sleeping => &[(2, 0), (2, 1)],
            Sprite::ScratchSelf => &[(5, 0), (6, 0), (7, 0)],
            Sprite::ScratchWall(Side::N) => &[(0, 0), (0, 1)],
            Sprite::ScratchWall(Side::S) => &[(7, 1), (6, 2)],
            Sprite::ScratchWall(Side::E) => &[(2, 2), (2, 3)],
            Sprite::ScratchWall(Side::W) => &[(4, 0), (4, 1)],
            Sprite::Walk(Dir::N) => &[(1, 2), (1, 3)],
            Sprite::Walk(Dir::NE) => &[(0, 2), (0, 3)],
            Sprite::Walk(Dir::E) => &[(3, 0), (3, 1)],
            Sprite::Walk(Dir::SE) => &[(5, 1), (5, 2)],
            Sprite::Walk(Dir::S) => &[(6, 3), (7, 2)],
            Sprite::Walk(Dir::SW) => &[(5, 3), (6, 1)],
            Sprite::Walk(Dir::W) => &[(4, 2), (4, 3)],
            Sprite::Walk(Dir::NW) => &[(1, 0), (1, 1)],
        }
    }

    /// Index into the frame list returned by `load_frames` for a given animation frame.
    pub fn frame_index(self, frame: usize) -> usize {
        let cells = self.cells();
        let (col, row) = cells[frame % cells.len()];
        (row * COLS + col) as usize
    }
}

/// Decode a character's sheet into 32 individual 32x32 RGBA frames, row-major.
pub fn load_frames(character: Character) -> Vec<RgbaImage> {
    let sheet = image::load_from_memory_with_format(character.sheet(), ImageFormat::Png)
        .expect("bundled sprite sheet is valid")
        .to_rgba8();
    split_sheet(sheet)
}

fn split_sheet(sheet: RgbaImage) -> Vec<RgbaImage> {
    assert_eq!((sheet.width(), sheet.height()), (COLS * CELL, ROWS * CELL));
    let mut frames = Vec::with_capacity((COLS * ROWS) as usize);
    for row in 0..ROWS {
        for col in 0..COLS {
            frames.push(image::imageops::crop_imm(&sheet, col * CELL, row * CELL, CELL, CELL).to_image());
        }
    }
    frames
}

/// Nearest-neighbour upscale of every frame to `px` square pixels.
pub fn scaled(frames: &[RgbaImage], px: u32) -> Vec<RgbaImage> {
    frames
        .iter()
        .map(|f| image::imageops::resize(f, px, px, FilterType::Nearest))
        .collect()
}

/// Tray icon from the idle frame. With `outline_only`, the white fill is made transparent
/// so the result works as a macOS template image.
pub fn tray_icon_rgba(frames: &[RgbaImage], outline_only: bool) -> (Vec<u8>, u32, u32) {
    let mut icon = frames[Sprite::Idle.frame_index(0)].clone();
    if outline_only {
        for p in icon.pixels_mut() {
            let [r, g, b, a] = p.0;
            if a > 0 && r > 128 && g > 128 && b > 128 {
                p.0 = [0, 0, 0, 0];
            }
        }
    }
    let icon = image::imageops::resize(&icon, 36, 36, FilterType::Nearest);
    (icon.into_raw(), 36, 36)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_character_sheet_decodes() {
        for c in Character::ALL {
            assert_eq!(load_frames(c).len(), 32, "{}", c.name());
        }
    }

    /// adryd retouched and nudged the walking frames in oneko.gif, but the static poses are
    /// untouched copies of the original XBMs. Those must match bit for bit, which pins both the
    /// XBM decoding and the cell layout used by scripts/xbm2sheet.py.
    #[test]
    fn converted_neko_matches_oneko_js_static_frames() {
        let gif = image::load_from_memory_with_format(include_bytes!("../assets/oneko.gif"), ImageFormat::Gif)
            .unwrap()
            .to_rgba8();
        let reference = split_sheet(gif);
        let ours = load_frames(Character::Neko);
        for sprite in [Sprite::Idle, Sprite::Alert, Sprite::Tired, Sprite::ScratchSelf] {
            let i = sprite.frame_index(2); // ScratchSelf frame 2 is jare2, the untouched one
            let (a, b) = (&reference[i], &ours[i]);
            let diff = a
                .pixels()
                .zip(b.pixels())
                .filter(|(p, q)| p.0[3] != q.0[3] || (p.0[3] != 0 && (p.0[0] > 128) != (q.0[0] > 128)))
                .count();
            assert_eq!(diff, 0, "{sprite:?} (frame {i}) differs from oneko.gif");
        }
    }

    #[test]
    fn sheet_decodes_with_transparency() {
        let frames = load_frames(Character::Neko);
        assert_eq!(frames.len(), 32);
        let idle = &frames[Sprite::Idle.frame_index(0)];
        assert_eq!(idle.get_pixel(0, 0).0[3], 0, "corner should be transparent");
        assert!(idle.pixels().any(|p| p.0[3] > 0), "idle frame should have opaque pixels");
    }
}
