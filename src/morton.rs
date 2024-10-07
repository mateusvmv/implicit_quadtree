
pub fn ordered_float(f: f32) -> u32 {
    let x = f.to_bits();
    if x & (1<<31) == 0 {
        x ^ (1<<31)
    } else {
        !x
    }
}
pub fn ordered_float_reverse(x: u32) -> f32 {
    if x & (1<<31) == 0 {
        f32::from_bits(!x)
    } else {
        f32::from_bits(x ^ (1<<31))
    }
}


fn spread_bits(mut input: u32) -> u64 {
    let mut output = 0;
    for _ in 0..32 {
        output = (output << 2) | (input & 1) as u64;
        input >>= 1;
    }
    output.reverse_bits()
}

fn collapse_bits(mut input: u64) -> u32 {
    let mut output = 0;
    for _ in 0..32 {
        output = (output << 1) | (input & 1) as u32;
        input >>= 2;
    }
    output.reverse_bits()
}

pub fn morton(x: u32, y: u32) -> u64 {
    spread_bits(x) | spread_bits(y) >> 1
}

pub fn morton_reverse(z: u64) -> (u32, u32) {
    (collapse_bits(z >> 1), collapse_bits(z))
}

const BIT_POSITION_INIT: u64 = 0x8000000000000000;
const LOAD_MASK_INIT: u64 = 0x5555555555555555;
const LOAD_ONES_INIT: u64 = 0x2aaaaaaaaaaaaaaa;
const ODDS: u64 = 0xaaaaaaaaaaaaaaaa;
const EVENS: u64 = 0x5555555555555555;

pub struct ZOrderIndexer {
    z: (u64, u64),
}

impl ZOrderIndexer {
    pub fn from_morton(min: u64, max: u64) -> Self {
        Self { z: (min, max) }
    }
    pub fn new(min: (u32, u32), max: (u32, u32)) -> Self {
        assert!(min.0 <= max.0 && min.1 <= max.1);
        let z = (morton(min.0, min.1), morton(max.0, max.1));
        assert!(z.0 <= z.1);
        Self { z }
    }
    pub fn from_float(min: (f32, f32), max: (f32, f32)) -> Self {
        let x = (ordered_float(min.0), ordered_float(min.1));
        let y = (ordered_float(max.0), ordered_float(max.1));
        Self::new(x, y)
    }
    pub fn from_aabb(range: &crate::AABB) -> Self {
        let min = (range.0.0 - range.1, range.0.1 - range.1);
        let max = (range.0.0 + range.1, range.0.1 + range.1);
        Self::from_float(min, max)
    }
    pub fn bounds(&self) -> &(u64, u64) {
        &self.z
    }
    pub fn contains(&self, z: u64) -> bool {
        let (x, y) = (z & ODDS, z & EVENS);
        x >= self.z.0 & ODDS && x <= self.z.1 & ODDS &&
            y >= self.z.0 & EVENS && y <= self.z.1 & EVENS
    }
    pub fn next_zorder_index(&self, z: u64) -> Option<u64> {
        if self.contains(z + 1) {
            return Some(z + 1);
        }
        let mut bigmin = None;
        let (mut min_v, mut max_v) = self.z;
        let mut bit_position = 1<<63;
        let mut load_mask = LOAD_MASK_INIT;
        let mut load_ones = LOAD_ONES_INIT;
        while bit_position > 0 {
            let k = ((z & bit_position > 0) as usize) << 2
                | ((min_v & bit_position > 0) as usize) << 1
                | (max_v & bit_position > 0) as usize;
            match k {
                0 => (),
                1 => {
                    bigmin = Some(min_v & load_mask | bit_position);
                    max_v = max_v & load_mask | load_ones;
                },
                3 => return Some(min_v),
                4 => return bigmin,
                5 => {
                    min_v = min_v & load_mask | bit_position;
                },
                7 => (),
                _ => unreachable!()
            }
            bit_position >>= 1;
            load_ones >>= 1;
            load_mask >>= 1;
            load_mask |= BIT_POSITION_INIT;
        }
        bigmin
    }
}
