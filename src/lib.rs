#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_4d;

mod morton;

pub use morton::*;

use std::collections::BTreeMap;

use rand::Rng;

pub fn generate_random_points(num_points: usize, size: f32) -> Vec<(f32, f32)> {
    let mut rng = rand::thread_rng();
    (0..num_points)
        .map(|_| (rng.gen_range(0.0..size), rng.gen_range(0.0..size)))
        .collect()
}

#[derive(Debug)]
pub struct QuadTree {
    tree: BTreeMap<u64, (f32, f32)>,
}

#[derive(Debug)]
pub struct AABB((f32, f32), f32);
impl AABB {
    pub fn contains(&self, point: &(f32, f32)) -> bool {
        let AABB(center, half_size) = *self;
        point.0 >= center.0 - half_size
            && point.0 <= center.0 + half_size
            && point.1 >= center.1 - half_size
            && point.1 <= center.1 + half_size
    }
}

impl QuadTree {
    pub fn new() -> Self {
        QuadTree {
            tree: BTreeMap::new(),
        }
    }
    pub fn insert(&mut self, point: (f32, f32)) {
        let x = ordered_float(point.0);
        let y = ordered_float(point.1);
        let z_index = morton_2(x, y);
        self.tree.insert(z_index, point);
    }

    pub fn query(&self, min: (u32, u32), max: (u32, u32)) -> impl Iterator<Item = &(f32, f32)> {
        let zi = ZOrderIndexer::<2>::new(min, max);
        let (min, max) = *zi.bounds();
        let mut cursor = self.tree.range(min ..= max);
        let mut missed = 0;
        std::iter::from_fn(move || {
            loop {
                let Some((k, p)) = cursor.next() else { break };
                if !zi.contains(*k) {
                    missed += 1;
                    if missed < 32 { continue };
                    let Some(k) = zi.next_zorder_index(*k) else { break };
                    cursor = self.tree.range(k ..= max);
                } else {
                    missed = 0;
                    return Some(p)
                }
            }
            None
        })
    }

    pub fn query_float(&self, min: (f32, f32), max: (f32, f32)) -> impl Iterator<Item = &(f32, f32)> {
        let x = (ordered_float(min.0), ordered_float(min.1));
        let y = (ordered_float(max.0), ordered_float(max.1));
        self.query(x, y)
    }

    pub fn query_aabb(&self, range: &AABB) -> impl Iterator<Item = &(f32, f32)> {
        let min = (range.0.0 - range.1, range.0.1 - range.1);
        let max = (range.0.0 + range.1, range.0.1 + range.1);
        self.query_float(min, max)
    }

    pub fn nearest(&self, point: (f32, f32)) -> impl Iterator<Item = &(f32, f32)> {
        let (x, y) = (ordered_float(point.0), ordered_float(point.1));
        let square_dist = move |p: (u32, u32)| u32::max(u32::abs_diff(p.0, x), u32::abs_diff(p.1, y));
        let square_dist = move |p: (f32, f32)| square_dist((ordered_float(p.0), ordered_float(p.1)));
        let z = morton_2(x, y);
        let mut a = self.tree.range(..z).rev()
            .map(move |(_, p)| (square_dist(*p), p))
            .peekable();
        let mut b = self.tree.range(z..)
            .map(move |(_, p)| (square_dist(*p), p))
            .peekable();
        let mut iter = std::iter::from_fn(move || match (a.peek(), b.peek()) {
            (None, None) => None,
            (Some(pa), Some(pb)) => if pa.0 <= pb.0 { a.next() } else { b.next() },
            (Some(_), None) => a.next(),
            (None, Some(_)) => b.next(),
        });
        let mut queue: Vec<&(f32, f32)> = (&mut iter).take_while(|t| t.0 == 0).map(|t| t.1).collect();
        let mut keys: Vec<u32> = Vec::new();
        let mut min_dist = 1;
        std::iter::from_fn(move || {
            loop {
                if let Some(p) = queue.pop() {
                    return Some(p);
                }
                let (distance, _) = (&mut iter).filter(|(d, _)| *d >= min_dist).take(8).max_by_key(|t| t.0)?;
                let squares = [
                    (
                        (x.saturating_sub(distance), y.saturating_sub(min_dist + 1)),
                        (x.saturating_sub(min_dist), y.saturating_add(min_dist - 1))
                    ),
                    (
                        (x.saturating_add(min_dist), y.saturating_sub(min_dist + 1)),
                        (x.saturating_add(distance), y.saturating_add(min_dist - 1))),
                    (
                        (x.saturating_sub(distance), y.saturating_sub(distance)),
                        (x.saturating_add(distance), y.saturating_sub(min_dist))),
                    (
                        (x.saturating_sub(distance), y.saturating_add(min_dist)),
                        (x.saturating_add(distance), y.saturating_add(distance))
                    ),
                ];
                let mut zis: Vec<_> = squares.into_iter()
                    .filter(|(min, max)| min.0 <= max.0 && min.1 <= max.1)
                    .map(|(min, max)| ZOrderIndexer::<2>::new(min, max))
                    .collect();
                let Some(min) = zis.iter().map(|zi| zi.bounds().0).min() else { continue };
                let Some(max) = zis.iter().map(|zi| zi.bounds().1).max() else { continue };
                let mut cursor = self.tree.range(min ..= max);
                let mut missed = 0;
                let mut zi = &zis[0];
                loop {
                    let Some((k, p)) = cursor.next() else { break };
                    if !zi.contains(*k) {
                        if zis.iter().any(|zi| zi.contains(*k)) {
                            zi = {
                                zis.retain(|zi| *k <= zi.bounds().1);
                                zis.iter().find(|zi| zi.contains(*k)).unwrap()
                            };
                        } else {
                            missed += 1;
                            if missed < 32 { continue };
                            let Some(k) = zis.iter()
                                .filter_map(|zi| zi.next_zorder_index(*k))
                                .min() else { break };
                            cursor = self.tree.range(k ..= max);
                            continue;
                        };
                    }
                    missed = 0;
                    queue.push(p);
                }
                if queue.len() < 64 {
                    keys.clear();
                    keys.extend(queue.iter().map(|&p| square_dist(*p)));
                    for i in 1..queue.len() {
                        for j in (0..i).rev() {
                            if keys[j] >= keys[j+1] { break };
                            keys.swap(j+1, j);
                            queue.swap(j+1, j);
                        }
                    }
                } else {
                    queue.sort_by_cached_key(|&p| std::cmp::Reverse(square_dist(*p)));
                }
                min_dist = distance + 1;
            }
        })
    }

    pub fn count_within_distance(&self, point: &(f32, f32), distance: f32) -> usize {
        let range = AABB(*point, distance);
        self.query_aabb(&range).count()
    }
}

impl Default for QuadTree {
    fn default() -> Self {
        Self::new()
    }
}
