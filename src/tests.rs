use super::*;

#[test]
fn test_indexer() {
    let zi = ZOrderIndexer::<2>::from_morton(0, 6);
    for i in 0..4 {
        assert!(zi.contains(i));
    }
    assert!(zi.contains(4));
    assert!(!zi.contains(5));
    assert!(zi.contains(6));
    assert!(!zi.contains(7));

    assert_eq!(zi.next_zorder_index(0), Some(1));
    assert_eq!(zi.next_zorder_index(1), Some(2));
    assert_eq!(zi.next_zorder_index(2), Some(3));
    assert_eq!(zi.next_zorder_index(3), Some(4));
    assert_eq!(zi.next_zorder_index(4), Some(6));
    assert_eq!(zi.next_zorder_index(5), Some(6));
    assert_eq!(zi.next_zorder_index(6), None);
}
#[test]
fn test_morton() {
    // Testing a range of values for x and y
    for x in 0..16 {
        for y in 0..16 {
            let z = morton_2(x, y);
            let (xr, yr) = morton_reverse_2(z);
            assert_eq!((x, y), (xr, yr), "Failed for (x={}, y={}): got (x={}, y={}) for (z={})", x, y, xr, yr, z);
        }
    }
}
#[test]
fn test_count_neighbors_correctness() {
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let num_points = 100;
        let distance = rng.gen_range(1e-1..5e1);

        // Generate random points
        let points = generate_random_points(num_points, 1e2);

        // Brute force count neighbors
        let mut brute_force_counts = Vec::new();
        for i in 0..num_points {
            let range = AABB(points[i], distance);
            let mut count = 0;
            for p in &points {
                if range.contains(p) { count += 1; }
            }
            brute_force_counts.push(count);
        }

        // Count neighbors using quadtree
        let quadtree_counts = {
            let mut quadtree = QuadTree::new();
            for point in &points {
                quadtree.insert(*point);
            }
            let mut counts = Vec::with_capacity(points.len());
            for point in &points {
                let count = quadtree.count_within_distance(point, distance);
                counts.push(count);
            }

            counts
        };

        // Check that both methods return the same result
        assert_eq!(brute_force_counts, quadtree_counts);
    }
}

#[test]
fn nns() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let num_points = rng.gen_range(100..1000);

        let mut tree = kdtree::KdTree::new(2);
        let mut quad = QuadTree::new();
        let points = generate_random_points(num_points, 1e2);
        for p in points {
            quad.insert(p);
            tree.add([p.0, p.1], p).unwrap();
        }
        fn square_dist(a: &[f32], b: &[f32]) -> f32 {
            let mut r: u32 = 0;
            for i in 0..a.len().min(b.len()) {
                r = r.max(u32::abs_diff(ordered_float(a[i]), ordered_float(b[i])));
            }
            r as f32
        }
        let a = tree.nearest(&[50.0, 50.0], 100, &square_dist).unwrap();
        let a: Vec<_> = a.into_iter().map(|t| t.1).collect();
        let b: Vec<_> = quad.nearest((50.0, 50.0)).take(100).collect();
        eprintln!("{:?}", a.iter().map(|p| square_dist(&[50.0, 50.0], &[p.0, p.1])).collect::<Vec<_>>());
        eprintln!("{:?}", b.iter().map(|p| square_dist(&[50.0, 50.0], &[p.0, p.1])).collect::<Vec<_>>());
        assert_eq!(a, b);
    }
}
