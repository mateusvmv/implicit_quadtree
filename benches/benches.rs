// benches/benches.rs

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use quadtree::{generate_random_points, QuadTree};
use rand::Rng;

fn benchmark_count_neighbors(c: &mut Criterion) {
    let sizes = [100, 1000, 10000, 20000];
    let mut cn = c.benchmark_group("count_neighbors");
    for size in sizes.iter().copied() {
        cn.bench_with_input(
            BenchmarkId::new("count_neighbors", size),
            &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut quadtree = QuadTree::new();
                for point in &points {
                    quadtree.insert(*point);
                }
                let mut rng = rand::thread_rng();
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    let dist = rng.gen_range(0.0..1e1);
                    quadtree.count_within_distance(&points[i], dist)
                }, BatchSize::SmallInput);
            }
        );
        cn.bench_with_input(
            BenchmarkId::new("count_neighbors_kd", size),
        &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut kdtree = kdtree::KdTree::new(2);
                for point in &points {
                    kdtree.add([point.0, point.1], ()).unwrap()
                }
                let mut rng = rand::thread_rng();
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    let dist = rng.gen_range(0.0..1e1);
                    kdtree.within(&[points[i].0, points[i].1], dist, &kdtree::distance::squared_euclidean)
                        .unwrap()
                        .len()
                }, BatchSize::SmallInput);
            }
        );
    }
    drop(cn);
    let mut knn = c.benchmark_group("10_nearest_neighbors");
    for size in sizes.iter().copied() {
        knn.bench_with_input(
            BenchmarkId::new("10_nearest_neighbors", size),
            &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut quadtree = QuadTree::new();
                for point in &points {
                    quadtree.insert(*point);
                }
                let mut rng = rand::thread_rng();
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    quadtree.nearest(points[i]).take(10).collect::<Vec<_>>()
                }, BatchSize::SmallInput);
            }
        );
        knn.bench_with_input(
            BenchmarkId::new("10_nearest_neighbors_kd", size),
        &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut kdtree = kdtree::KdTree::new(2);
                for point in &points {
                    kdtree.add([point.0, point.1], ()).unwrap()
                }
                let mut rng = rand::thread_rng();
                fn chebyshev(a: &[f32], b: &[f32]) -> f32 {
                    let mut r: f32 = 0.0;
                    for i in 0..a.len().min(b.len()) {
                        r = r.max((a[i] - b[i]).abs());
                    }
                    r
                }
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    kdtree.nearest(&[points[i].0, points[i].1], 10, &chebyshev)
                        .unwrap()
                        .len()
                }, BatchSize::SmallInput);
            }
        );
    }
    drop(knn);
    let mut knn = c.benchmark_group("k_nearest_neighbors");
    knn.warm_up_time(Duration::from_millis(100));
    knn.measurement_time(Duration::from_millis(200));
    let size = 50000;
    for k in sizes.iter().copied() {
        knn.bench_with_input(
            BenchmarkId::new("k_nearest_neighbors", k),
            &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut quadtree = QuadTree::new();
                for point in &points {
                    quadtree.insert(*point);
                }
                let mut rng = rand::thread_rng();
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    quadtree.nearest(points[i]).take(k).collect::<Vec<_>>()
                }, BatchSize::SmallInput);
            }
        );
        knn.bench_with_input(
            BenchmarkId::new("k_nearest_neighbors_kd", k),
        &size,
            |b, &size| {
                let points = generate_random_points(size, 1e6);
                let mut kdtree = kdtree::KdTree::new(2);
                for point in &points {
                    kdtree.add([point.0, point.1], ()).unwrap()
                }
                let mut rng = rand::thread_rng();
                fn chebyshev(a: &[f32], b: &[f32]) -> f32 {
                    let mut r: f32 = 0.0;
                    for i in 0..a.len().min(b.len()) {
                        r = r.max((a[i] - b[i]).abs());
                    }
                    r
                }
                b.iter_batched(|| (), |()| {
                    let i = rng.gen_range(0..points.len());
                    kdtree.nearest(&[points[i].0, points[i].1], k, &chebyshev)
                        .unwrap()
                        .len()
                }, BatchSize::SmallInput);
            }
        );
    }
}

criterion_group!(benches, benchmark_count_neighbors);
criterion_main!(benches);

