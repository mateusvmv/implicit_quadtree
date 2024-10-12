# Data Structure
This data structure has many names, it is essentially a BTree which uses Morton Code for its indices. You can call it Morton Code BTree, QuadTree on a BTree, Z-Order BTree, I think it looks feels a lot like implicit data structures, which are implemented on top of other data structures (arrays) for better constants, so I named this repository implicit QuadTree.

# Implementation
This implementation provides methods for range queries and nearest neighbor searches using a BTree and Morton Code indexes. I compare it with the kdtree crate for benchmarks, and it's pretty good! Range queries are faster (although I'll admit the benchmark is rigged because their API is vector based and not iterator based for range queries). The picture is different with all nearest neighbor searches, and this tree implementation was slower by a constant factor of 5 on my machine.

## Implementation details
There are functions that turn floats into unsigned integers while maintaing their order, and also the morton order conversions, which you will find in the morton.rs file. The conversion to morton is first spreading the bits out and then doing an or for each one of the two dimensions. The CPU seemed to like it that way (it was faster) so I kept it like that. There is a struct which serves the sole purpose of calculating the next z-order index (analogous to the next quadtree segment, maybe) which falls within a quadrant which I translated from the [pyzorder](https://github.com/smatsumt/pyzorder) library, it is the same algorithm in the [this paper](https://www.vision-tools.com/fileadmin/unternehmen/HTR/DBCode_mit_Erlaeuterung.txt)

## The Nearest Neighbor Search
The nearest neighbor search is looking for neighbors in the curve, and then doing 4 range queries on the 2d space to assert that there are no neighbors closer to our origin point. It searches both forwards and backwards, and always picks the closest neighbor in curve, so that might protect it against the edge case of z-order curves. I don't think it is very good for nearest neighbor searches though, it piggybacks on the superb performance of the range bounds query to have an acceptable performance.

## 4D Rectangle Collision Problem
The rectangle collision problem, which is normally thought of in 2 dimensions, is actually a 4D range query. Suppose you have a query rectangle (a, b, c, d), where (a, b) is the top left point, and (c, d) is the bottom right point, if you want to find out which rectangles are colliding with it, you'd do a range query in 4D, to find all rectangles (e, f, g, h) for which (0, 0, a, b) <= (e, f, g, h) <= (c, d, inf, inf). That is, the other rectangle begins before the query rectangle ends, and the other rectangle ends after the query rectangle begins. The performance of a Z-Order BTree is on par, if not better, than an RTree, for this specific problem.

# Use cases
A Z-Index BTree has better memory locality and benefits from all optimizations applied to BTrees, which KDTrees and QuadTrees do not benefit from. That also means that it is a good structure to slap on the disk! Just grab lmdb and have fun, if you ever need that to persist a game save file and load it without serialization or something analogous to that.

## It's simple
It's such a simple data structure, and also flexible. You can use it with any sorted container, like, for example the [log² vector of sorted vectors](https://www.nayuki.io/page/binary-array-set), a dead simple sorted container if you don't want to use, implement, or can't have BTrees.

# Benchmarks
Benchmarks can be ran using `> cargo bench`
