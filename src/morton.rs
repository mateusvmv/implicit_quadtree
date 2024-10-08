
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

const X_DIM: u64 = 0xAAAAAAAAAAAAAAAA;
const Y_DIM: u64 = 0x5555555555555555;

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
        [X_DIM, Y_DIM].iter()
            .all(|dim| 
                z & dim >= self.z.0 & dim &&
                z & dim <= self.z.1 & dim)
    }
    pub fn next_zorder_index(&self, z: u64) -> Option<u64> {
        if self.contains(z + 1) {
            return Some(z + 1);
        }
        let mut bigmin = None;
        let (mut min_v, mut max_v) = self.z;
        // One in the current dimension, and in the past bits (to the left)
        let mut load_mask = Y_DIM;
        // One in all dimensions but the current one, except in the past bits, which are zero
        let mut load_ones = X_DIM;
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
