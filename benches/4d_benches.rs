use std::{collections::BTreeMap, time::Duration};

use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use quadtree::{morton_4, ZOrderIndexer};
use rand::Rng;
use rstar::{primitives::Rectangle, RTree, RTreeObject};

fn generate_random_4d_points(num_points: usize, bounds: std::ops::Range<i32>) -> Vec<(i32, i32, i32, i32)> {
    let mut rng = rand::thread_rng();
    (0..num_points).map(|_| {
        (rng.gen_range(bounds.clone()), rng.gen_range(bounds.clone()), 
         rng.gen_range(bounds.clone()), rng.gen_range(bounds.clone()))
    }).collect()
}

// Example of a brute force rectangle intersection check in 4D
fn brute_force_range_query(
    points: &[(i32, i32, i32, i32)], 
    x_range: std::ops::Range<i32>, 
    y_range: std::ops::Range<i32>, 
    z_range: std::ops::Range<i32>, 
    w_range: std::ops::Range<i32>
) -> Vec<(i32, i32, i32, i32)> {
    points.iter().cloned().filter(|p| {
        x_range.contains(&p.0) && y_range.contains(&p.1) && z_range.contains(&p.2) && w_range.contains(&p.3)
    }).collect()
}

const BOUNDS: std::ops::Range<i32> = 0..32_000;

// Benchmark for brute force approach
fn bruteforce_benchmark(b: &mut Bencher<'_>, num_points: usize, rect_size: i32) {
    let mut rng = rand::thread_rng();
    let points = generate_random_4d_points(num_points, BOUNDS);

    
    b.iter(|| {
        let mut rect = (rng.gen_range(0..BOUNDS.end), rng.gen_range(0..BOUNDS.end), 0, 0);
        rect.2 = (rect.0 + rect_size).min(BOUNDS.end);
        rect.3 = (rect.1 + rect_size).min(BOUNDS.end);
        let x_range = 0..rect.2;
        let y_range = 0..rect.3;
        let z_range = rect.0..BOUNDS.end;
        let w_range = rect.1..BOUNDS.end;
        brute_force_range_query(&points, x_range.clone(), y_range.clone(), z_range.clone(), w_range.clone())
    });
}

// Benchmark for Z-order based approach
fn z_order_benchmark(b: &mut Bencher<'_>, num_points: usize, rect_size: i32) {
    let mut rng = rand::thread_rng();
    let points = generate_random_4d_points(num_points, BOUNDS);

    // Create a Morton-encoded Z-order tree
    let mut z_ordered_points: Vec<_> = points.clone();
    z_ordered_points.sort_by_key(|p| morton_4(p.0 as u16, p.1 as u16, p.2 as u16, p.3 as u16));
    let key: Vec<_> = z_ordered_points.iter().map(|p| morton_4(p.0 as u16, p.1 as u16, p.2 as u16, p.3 as u16)).collect();
    
    b.iter(|| {
        let mut rect = (rng.gen_range(0..BOUNDS.end), rng.gen_range(0..BOUNDS.end), 0, 0);
        rect.2 = (rect.0 + rect_size).min(BOUNDS.end) as u16;
        rect.3 = (rect.1 + rect_size).min(BOUNDS.end) as u16;
        let min = morton_4(0, 0, rect.0 as u16, rect.1 as u16);
        let max = morton_4(rect.2, rect.3, BOUNDS.end as u16, BOUNDS.end as u16);
        let zi = ZOrderIndexer::<4>::from_morton(min, max); // Assuming this is implemented

        let end_idx = key.partition_point(|&k| k <= max);
        let mut results = Vec::new();
        let mut i = 0;
        let mut misses = 0;
        while i < end_idx {
            let p = &z_ordered_points[i];
            let z = key[i];
            i += 1;
            if !zi.contains(z) {
                misses += 1;
                if misses < 32 { continue };
                let Some(z) = zi.next_zorder_index(z) else { break };
                i += key[i..end_idx].partition_point(|&k| k < z);
            } else {
                misses = 0;
                results.push(p);
            }
        }
        results
    });
}

// Benchmark for Z-order based approach in BTree
fn z_order_btree_benchmark(b: &mut Bencher<'_>, num_points: usize, rect_size: i32) {
    let mut rng = rand::thread_rng();
    let points = generate_random_4d_points(num_points, BOUNDS);

    // Create a Morton-encoded Z-order tree
    let z_ordered_points: BTreeMap<_, _> = points
        .into_iter()
        .map(|p| (morton_4(p.0 as u16, p.1 as u16, p.2 as u16, p.3 as u16), p))
        .collect();

    b.iter(|| {
        let mut rect = (rng.gen_range(0..BOUNDS.end), rng.gen_range(0..BOUNDS.end), 0, 0);
        rect.2 = (rect.0 + rect_size).min(BOUNDS.end) as u16;
        rect.3 = (rect.1 + rect_size).min(BOUNDS.end) as u16;
        let min = morton_4(0, 0, rect.0 as u16, rect.1 as u16);
        let max = morton_4(rect.2, rect.3, BOUNDS.end as u16, BOUNDS.end as u16);
        let zi = ZOrderIndexer::<4>::from_morton(min, max); // Assuming this is implemented

        let mut cursor = z_ordered_points.range(min..=max);
        let mut results = Vec::new();
        let mut misses = 0;
        loop {
            let Some((&z, p)) = cursor.next() else { break };
            if !zi.contains(z) {
                misses += 1;
                if misses < 32 { continue };
                let Some(z) = zi.next_zorder_index(z) else { break };
                cursor = z_ordered_points.range(z..=max);
            } else {
                misses = 0;
                results.push(p);
            }
        }
        results
    });
}

// Benchmark for R-Tree approach
fn r_tree_benchmark(b: &mut Bencher<'_>, num_points: usize, rect_size: i32) {
    let mut rng = rand::thread_rng();
    let points = generate_random_4d_points(num_points, BOUNDS);
    let rtree: RTree<Rectangle<(i32, i32)>> = RTree::bulk_load(points
        .iter()
        .map(|p| Rectangle::from_corners((p.0, p.1), (p.2, p.3)))
        .collect());
    
    b.iter(|| {
        let mut rect = (rng.gen_range(0..BOUNDS.end), rng.gen_range(0..BOUNDS.end), 0, 0);
        rect.2 = (rect.0 + rect_size).min(BOUNDS.end);
        rect.3 = (rect.1 + rect_size).min(BOUNDS.end);
        let rect = Rectangle::from_corners((rect.0, rect.1), (rect.2, rect.3));

        let results: Vec<_> = rtree.locate_in_envelope(&rect.envelope()).collect();
        results
    });
}

// Setting up Criterion benchmark group
fn criterion_benchmark(c: &mut Criterion) {
    let mut c = c.benchmark_group("4d rectangle query");
    c.warm_up_time(Duration::from_millis(100));
    c.measurement_time(Duration::from_millis(200));
    let size = 100_000;
    for rect_size in [6000, 9000, 12000, 15000] {
        c.bench_with_input(BenchmarkId::new("bruteforce", rect_size), &(size, rect_size), |c, &(n, r)| bruteforce_benchmark(c, n, r));
        c.bench_with_input(BenchmarkId::new("z_ord_vec", rect_size), &(size, rect_size), |c, &(n, r)| z_order_benchmark(c, n, r));
        c.bench_with_input(BenchmarkId::new("z_ord_bt", rect_size), &(size, rect_size), |c, &(n, r)| z_order_btree_benchmark(c, n, r));
        c.bench_with_input(BenchmarkId::new("r_tree", rect_size), &(size, rect_size), |c, &(n, r)| r_tree_benchmark(c, n, r));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
