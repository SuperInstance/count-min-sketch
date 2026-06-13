# Count-Min Sketch

**A sublinear-space probabilistic data structure for frequency estimation.** The Count-Min Sketch (CMS) tracks how many times items have been seen in a data stream using a fraction of the memory required by an exact counter, with mathematically bounded error.

## Why It Matters

Exact frequency counting requires O(n) space — one entry per distinct item. For high-cardinality streams (network packets, search queries, click events), this is infeasible. The Count-Min Sketch, introduced by Cormode and Muthukrishnan (2003), reduces space to O(1/ε · log(1/δ)) while guaranteeing that the estimated count is within ε of the true count with probability 1−δ.

**Key guarantee:** The estimate is always ≥ the true count (never under-counts), and exceeds it by at most ε·N (where N is total items seen) with probability 1−δ. This one-sided error makes CMS ideal for:

- **Network traffic analysis** — counting packet flows at line rate
- **Stream processing** — top-K frequent items, heavy hitters
- **Database query optimization** — estimating join sizes and selectivity
- **CDN edge caching** — tracking content popularity with fixed memory

The space savings are dramatic: tracking 1 billion distinct items to within 1% error at 99% confidence requires only ~5 KB of counters.

## How It Works

A Count-Min Sketch is a 2D array of counters with `depth` rows and `width` columns. Each row is associated with an independent hash function.

**Insertion:** To add an item, hash it with each row's hash function to get `depth` column indices, then increment the counter at each position. This means each item touches `depth` counters across the table.

**Query:** To estimate an item's frequency, hash it to find the same `depth` positions and return the **minimum** counter value. The minimum is used because hash collisions can only inflate counts (never deflate them), so the row with the fewest collisions gives the tightest estimate.

**Why the minimum works:** If item A and item B hash to the same cell in row 3, that cell's count = count(A) + count(B) ≥ count(A). But across `depth` independent hash functions, the probability that B collides with A in *every* row drops exponentially. Taking the minimum across rows exploits this: at least one row is likely collision-free for A.

**Parameter derivation:** Given error bound ε and failure probability δ:
- `width = ⌈e/ε⌉` (Euler's number divided by error)
- `depth = ⌈ln(1/δ)⌉` (logarithm of inverse confidence)

The implementation uses double-hashing (salting the default hasher with a row-dependent constant) rather than maintaining `depth` independent hash functions, which is a standard space optimization.

## Quick Start

```rust
use count_min_sketch::CountMinSketch;

// Create a sketch with ε=0.01, δ=0.01
// → width ≈ 272, depth ≈ 5
// Error ≤ 1% of total count, 99% of the time
let mut cms = CountMinSketch::new(0.01, 0.01);

// Count items in a stream
for _ in 0..1000 {
    cms.insert(&"GET /api/users");
}
for _ in 0..250 {
    cms.insert(&"POST /api/login");
}

// Query frequencies (always ≥ true count)
assert!(cms.count(&"GET /api/users") >= 1000);
println!("Estimated count: {}", cms.count(&"GET /api/users"));
println!("Memory used: {} bytes", cms.memory_bytes());

// Inner product estimation (for self-join size estimation)
let ip = cms.inner_product(&"GET /api/users", &"POST /api/login");
```

## API

### `CountMinSketch`
- `new(epsilon: f64, delta: f64) -> Self` — Create with error ε and failure probability δ
- `with_dimensions(width: usize, depth: usize) -> Self` — Create with explicit table size
- `insert<T: Hash>(&mut self, item: &T)` — Add item with count 1
- `add<T: Hash>(&mut self, item: &T, n: u64)` — Add item with arbitrary count
- `count<T: Hash>(&self, item: &T) -> u64` — Estimate frequency (always ≥ true count). O(depth)
- `inner_product<T: Hash>(&self, a: &T, b: &T) -> u64` — Estimate inner product of two item sets
- `clear(&mut self)` — Reset all counters to zero. O(width × depth)
- `memory_bytes(&self) -> usize` — Total memory consumption
- `width(&self) -> usize` / `depth(&self) -> usize` — Dimensions

## Architecture Notes

This is a foundational data structure within the SuperInstance analytics layer, used for real-time stream frequency estimation where memory is constrained and approximate counts are acceptable.

See the full architecture: [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md)

## License

MIT
