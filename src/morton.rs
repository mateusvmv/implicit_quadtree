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

fn spread_bits_2(mut input: u32) -> u64 {
    let mut output = 0;
    for _ in 0..32 {
        output = (output << 2) | (input & 1) as u64;
        input >>= 1;
    }
    output.reverse_bits()
}

fn collapse_bits_2(mut input: u64) -> u32 {
    let mut output = 0;
    for _ in 0..32 {
        output = (output << 1) | (input & 1) as u32;
        input >>= 2;
    }
    output.reverse_bits()
}

pub fn morton_2(x: u32, y: u32) -> u64 {
    spread_bits_2(x) | spread_bits_2(y) >> 1
}

pub fn morton_reverse_2(z: u64) -> (u32, u32) {
    (collapse_bits_2(z >> 1), collapse_bits_2(z))
}


fn spread_bits_4(mut input: u16) -> u64 {
    let mut output = 0;
    for _ in 0..16 {
        output = (output << 4) | (input & 1) as u64;
        input >>= 1;
    }
    output.reverse_bits()
}

fn collapse_bits_4(mut input: u64) -> u16 {
    let mut output = 0;
    for _ in 0..16 {
        output = (output << 1) | (input & 1) as u16;
        input >>= 4;
    }
    output.reverse_bits()
}

pub fn morton_4(x: u16, y: u16, z: u16, w: u16) -> u64 {
    spread_bits_4(x) |
    spread_bits_4(y) >> 1 |
    spread_bits_4(z) >> 2 |
    spread_bits_4(w) >> 3
}

pub fn morton_reverse_4(z: u64) -> (u16, u16, u16, u16) {
    (
        collapse_bits_4(z >> 3),
        collapse_bits_4(z >> 2),
        collapse_bits_4(z >> 1),
        collapse_bits_4(z)
    )
}

const fn dim_masks<const D: usize>() -> [u64; D] {
    let mut masks = [0; D];
    masks[D-1] = 1;
    let mut shf = D;
    while shf < 64 {
        masks[D-1] |= masks[D-1] << shf;
        shf *= 2;
    }
    let mut i = D-1;
    while i > 0 {
        masks[i-1] = masks[i] << 1;
        i -= 1;
    }
    masks
}

pub struct ZOrderIndexer<const D: usize>((u64, u64));

type Point16 = (u16, u16);
type Rect16 = (Point16, Point16);

impl ZOrderIndexer<2> {
    pub fn new(min: (u32, u32), max: (u32, u32)) -> Self {
        let min = morton_2(min.0, min.1);
        let max = morton_2(max.0, max.1);
        Self::from_morton(min, max)
    }
}

impl ZOrderIndexer<4> {
    pub fn new(min: Rect16, max: Rect16) -> Self {
        let min = morton_4(min.0.0, min.0.1, min.1.0, min.1.1);
        let max = morton_4(max.0.0, max.0.1, max.1.0, max.1.1);
        Self::from_morton(min, max)
    }
}
impl<const D: usize> ZOrderIndexer<D> {
    const DIMS: [u64; D] = dim_masks::<D>();
    pub fn from_morton(min: u64, max: u64) -> Self {
        assert!(Self::DIMS.iter().all(|dim| min & dim <= max & dim));
        Self((min, max))
    }
    pub fn bounds(&self) -> &(u64, u64) {
        &self.0
    }
    pub fn contains(&self, z: u64) -> bool {
        Self::DIMS.iter().all(|dim|
            z & dim >= self.0.0 & dim &&
            z & dim <= self.0.1 & dim)
    }
    pub fn next_zorder_index(&self, z: u64) -> Option<u64> {
        if self.contains(z + 1) {
            return Some(z + 1);
        }
        let mut bigmin = None;
        let (mut min_v, mut max_v) = self.0;
        // One in all dimensions but the current one, and in all past bits
        // Preserves the value of those bits, zeros the current dimension
        let mut load_mask = !Self::DIMS[0];
        // One in the current dimension, except past bits
        let mut load_ones = Self::DIMS[0] >> D;
        // Each bit draws an axis in some dimension, that we use to narrow down our search space
        for bit in (0..64).rev() {
            let z_bit = z >> bit & 1;
            let i_bit = min_v >> bit & 1;
            let a_bit = max_v >> bit & 1;
            match (z_bit, i_bit, a_bit) {
                // If all values are before the axis, we do nothing
                (0, 0, 0) => (),
                // If our target is before and the max is after
                // We set our candidate to be the first value after the axis
                // And move our search bounds to be before the axis
                (0, 0, 1) => {
                    bigmin = Some(min_v & load_mask | (1<<bit));
                    max_v = max_v & load_mask | load_ones;
                },
                // If our target is before the search area,
                // the result the minimum of the search area.
                (0, 1, 1) => return Some(min_v),
                // If our target is after the search area,
                // the result is the candidate, the first value within the area
                (1, 0, 0) => return bigmin,
                // If our target is after and our min is before
                // We move our search bounds to after the axis
                // We don't set a candidate, because it would be before the target
                (1, 0, 1) => {
                    min_v = min_v & load_mask | (1<<bit);
                },
                // If all values are past the axis, we do nothing
                (1, 1, 1) => (),
                _ => unreachable!()
            }
            load_ones >>= 1;
            load_mask >>= 1;
            load_mask |= 1<<63;
        }
        bigmin
    }
}
