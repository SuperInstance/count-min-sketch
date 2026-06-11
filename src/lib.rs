//! Count-Min Sketch — a probabilistic data structure for frequency estimation.

use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone)]
pub struct CountMinSketch {
    table: Vec<Vec<u64>>,
    width: usize,
    depth: usize,
}

impl CountMinSketch {
    /// Create a new Count-Min Sketch.
    /// * `epsilon` — error factor (width ≈ ⌈e/ε⌉)
    /// * `delta` — probability of exceeding error (depth ≈ ⌈ln(1/δ)⌉)
    pub fn new(epsilon: f64, delta: f64) -> Self {
        assert!(epsilon > 0.0 && epsilon < 1.0, "epsilon must be in (0,1)");
        assert!(delta > 0.0 && delta < 1.0, "delta must be in (0,1)");
        let width = (std::f64::consts::E / epsilon).ceil() as usize;
        let depth = (1.0 / delta).ln().ceil() as usize;
        let table = vec![vec![0u64; width]; depth];
        Self { table, width, depth }
    }

    /// Create with explicit dimensions.
    pub fn with_dimensions(width: usize, depth: usize) -> Self {
        let table = vec![vec![0u64; width]; depth];
        Self { table, width, depth }
    }

    fn hash_pair_index(&self, item: &impl Hash, row: usize) -> usize {
        let mut h1 = DefaultHasher::new();
        item.hash(&mut h1);
        let v = h1.finish();
        let salted = v.wrapping_add((row as u64).wrapping_mul(0x9e3779b97f4a7c15));
        let mut h2 = DefaultHasher::new();
        salted.hash(&mut h2);
        h2.finish() as usize % self.width
    }

    /// Insert an item with count `n`.
    pub fn add<T: Hash>(&mut self, item: &T, n: u64) {
        for row in 0..self.depth {
            let col = self.hash_pair_index(item, row);
            self.table[row][col] += n;
        }
    }

    /// Insert an item once.
    pub fn insert<T: Hash>(&mut self, item: &T) {
        self.add(item, 1);
    }

    /// Query the estimated frequency of an item (always ≥ true count).
    pub fn count<T: Hash>(&self, item: &T) -> u64 {
        (0..self.depth)
            .map(|row| {
                let col = self.hash_pair_index(item, row);
                self.table[row][col]
            })
            .min()
            .unwrap_or(0)
    }

    /// Estimate inner product of two item sets.
    pub fn inner_product<T: Hash>(&self, a: &T, b: &T) -> u64 {
        (0..self.depth)
            .map(|row| {
                let ca = self.hash_pair_index(a, row);
                let cb = self.hash_pair_index(b, row);
                self.table[row][ca].min(self.table[row][cb])
            })
            .min()
            .unwrap_or(0)
    }

    /// Reset all counters.
    pub fn clear(&mut self) {
        for row in &mut self.table {
            row.fill(0);
        }
    }

    /// Memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.width * self.depth * std::mem::size_of::<u64>()
    }

    pub fn width(&self) -> usize { self.width }
    pub fn depth(&self) -> usize { self.depth }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_count() {
        let mut cms = CountMinSketch::with_dimensions(1000, 5);
        for _ in 0..100 {
            cms.insert(&"hello");
        }
        let c = cms.count(&"hello");
        assert_eq!(c, 100);
    }

    #[test]
    fn test_approximate() {
        let mut cms = CountMinSketch::new(0.01, 0.01);
        for i in 0..10000u64 {
            for _ in 0..(i % 10 + 1) {
                cms.insert(&i);
            }
        }
        for i in 0..10000u64 {
            let true_count = (i % 10 + 1) as u64;
            let est = cms.count(&i);
            assert!(est >= true_count, "under-count for {i}");
            assert!(est <= true_count * 3, "over-count for {i}: {est}");
        }
    }
}
