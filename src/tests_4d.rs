extern crate rand;

use rand::Rng;
use std::ops::Range;

// A 4D point is represented as a tuple (x, y, z, w)
type Point4D = (i32, i32, i32, i32);

// Generates random 4D points within a specified range.
fn generate_random_points(count: usize, bounds: Range<i32>) -> Vec<Point4D> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            (
                rng.gen_range(bounds.clone()), // x
                rng.gen_range(bounds.clone()), // y
                rng.gen_range(bounds.clone()), // z
                rng.gen_range(bounds.clone()), // w
            )
        })
        .collect()
}

// Checks if a given 4D point is within the specified range in each dimension.
fn is_point_in_range(point: &Point4D, x_range: &Range<i32>, y_range: &Range<i32>, z_range: &Range<i32>, w_range: &Range<i32>) -> bool {
    x_range.contains(&point.0) &&
    y_range.contains(&point.1) &&
    z_range.contains(&point.2) &&
    w_range.contains(&point.3)
}

// Performs a brute-force search for all points within the specified 4D range.
fn brute_force_range_query(points: &[Point4D], x_range: Range<i32>, y_range: Range<i32>, z_range: Range<i32>, w_range: Range<i32>) -> Vec<Point4D> {
    points
        .iter()
        .filter(|&&point| is_point_in_range(&point, &x_range, &y_range, &z_range, &w_range))
        .cloned()
        .collect()
}

#[test]
fn test_4d_range_query() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let num_points = 1000;
        let bounds = 0..100; // Points will have coordinates between 0 and 1000
        let points = generate_random_points(num_points, bounds.clone());
    
        // Generate a random 4D range
        let mut gen_range = || {
            let from = rng.gen_range(bounds.clone());
            let length = rng.gen_range(bounds.clone());
            from..(from+length+1)
        };
        let x_range = gen_range();
        let y_range = gen_range();
        let z_range = gen_range();
        let w_range = gen_range();
    
        // Perform a brute-force search
        let brute_force_results = brute_force_range_query(&points, x_range.clone(), y_range.clone(), z_range.clone(), w_range.clone());
    
        let mkm = |p: &(i32, i32, i32, i32)| morton_4(p.0 as u16, p.1 as u16, p.2 as u16, p.3 as u16);
        // Call your Z-order range query code here and compare the results:
        let mut tree = points.clone();
        tree.sort_by_key(mkm);
        let tree = tree;
    
        use crate::morton::*;
        let min = morton_4(x_range.start as u16, y_range.start as u16, z_range.start as u16, w_range.start as u16);
        let max = morton_4(x_range.end as u16 - 1, y_range.end as u16 - 1, z_range.end as u16 - 1, w_range.end as u16 - 1);
        let zi = ZOrderIndexer::<4>::from_morton(min, max);

        let end_idx = tree.partition_point(|p| mkm(p) <= max);
        let mut results = Vec::new();
        let mut i = 0;
        while i < end_idx {
            let p = &tree[i];
            i += 1;
            let z = mkm(p);
            if !zi.contains(z) {
                let Some(z) = zi.next_zorder_index(z) else { break };
                i += tree[i..end_idx].partition_point(|p| mkm(p) < z);
            } else {
                results.push(p);
            }
        }
    
        // You can compare the results like this:
        assert_eq!(brute_force_results.len(), results.len());
    }
}
