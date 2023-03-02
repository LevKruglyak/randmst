use std::{
    cell::Cell,
    ops::{Index, IndexMut},
};

use rand_distr::{Distribution, Uniform};
use smallvec::SmallVec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Point(u32);

impl Point {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct LinkSizeCompact {
    data: Cell<u32>,
}

pub enum LinkSize {
    Parent(Point),
    Size(u32),
}

impl LinkSizeCompact {
    const SENTINEL: u32 = 1 << 31;
    const SENTINEL_MASK: u32 = !Self::SENTINEL;

    fn root(size: u32) -> Self {
        debug_assert_eq!(size & Self::SENTINEL, 0);
        Self {
            data: Cell::new(Self::SENTINEL | size),
        }
    }

    fn link(parent: Point) -> Self {
        debug_assert_eq!(parent.0 & Self::SENTINEL_MASK, parent.0);
        Self {
            data: Cell::new(parent.0),
        }
    }

    fn set(&self, u: &Self) {
        self.data.set(u.data.get())
    }

    fn is_root(&self) -> bool {
        self.data.get() & Self::SENTINEL != 0
    }

    fn expand(&self) -> LinkSize {
        if self.is_root() {
            LinkSize::Size(self.data.get() & Self::SENTINEL_MASK)
        } else {
            LinkSize::Parent(Point(self.data.get()))
        }
    }

    fn try_size(&self) -> Option<u32> {
        match self.expand() {
            LinkSize::Size(size) => Some(size),
            _ => None,
        }
    }

    fn add_size(&mut self, size: u32) {
        debug_assert!(self.is_root());
        self.data.set(self.data.get() + size);
    }
}

pub struct SizedUnionFind {
    /// Sends each Point (as a usize) to its Parent. (roots are self-parent)
    data: Vec<LinkSizeCompact>,

    // Metadata to keep track of sizes for sampling purposes
    size: u32,
    total_edges: usize,
    total_internal: usize,

    // Marginal speedup when storing distribution object
    vertex_distr: Uniform<u32>,
}

impl SizedUnionFind {
    pub fn new(size: u32) -> Self {
        assert_eq!(size & LinkSizeCompact::SENTINEL, 0);
        Self {
            data: (0..size)
                .into_iter()
                .map(|_| LinkSizeCompact::root(1))
                .collect(),
            size,
            total_edges: (size as usize) * (size as usize - 1) / 2,
            total_internal: 0,
            vertex_distr: Uniform::new(0, size),
        }
    }

    /// Unites the two sets, returns the size and root of the united
    /// component. Returns `None` if these two points are in the same component
    pub fn unite(&mut self, mut u: Point, mut v: Point) -> bool {
        // Do aggressive path compression during unite operation
        let mut queue: SmallVec<[Point; 4]> = SmallVec::new();

        while self.parent(u) != self.parent(v) {
            // Make sure we're oriented properly to keep root nodes
            // at the end of the array
            if self.parent(u) > self.parent(v) {
                std::mem::swap(&mut u, &mut v);
            }

            // If we've reached a root, we're done
            if let Some(join_size) = self[u].try_size() {
                self.link(u, self.parent(v));

                let (root, size) = self.root_size(v);
                self[root].set(&LinkSizeCompact::root(size + join_size));

                for p in queue {
                    self.link(p, root);
                }

                // Update total internal edges
                self.total_internal += size as usize * join_size as usize;

                return true;
            }

            let temp = self.parent(u);
            queue.push(u);
            // self.link(u, self.parent(v));
            u = temp;
        }

        // This means that these points were already part of the same component
        false
    }

    /// Are these two points in the same set?
    pub fn same_set(&self, mut u: Point, mut v: Point) -> bool {
        // TODO: see if unrolling is faster
        self.root(u) == self.root(v)
    }

    pub fn root(&self, u: Point) -> Point {
        self.root_size(u).0
    }

    pub fn size(&self, u: Point) -> u32 {
        self.root_size(u).1
    }

    pub fn root_size(&self, mut u: Point) -> (Point, u32) {
        let mut root = u;
        if let Some(size) = self[root].try_size() {
            return (root, size);
        }

        let mut prev = &self[root];

        loop {
            let temp = self.parent(root);

            if let Some(size) = self[temp].try_size() {
                return (temp, size);
            }

            // Path splitting
            prev.set(&self[temp]);
            prev = &self[temp];
            root = temp;
        }
    }

    fn link(&self, src: Point, dst: Point) {
        // Important step to avoid infinite sentinel
        if src != dst {
            self[src].set(&LinkSizeCompact::link(dst));
        }
    }

    fn parent(&self, u: Point) -> Point {
        match self[u].expand() {
            LinkSize::Parent(parent) => parent,
            LinkSize::Size(_) => u,
        }
    }

    /// Returns the number of internal edges among the components
    pub fn linked_edges(&self) -> usize {
        self.total_internal
    }

    /// Returns the number of non-internal edges among the components
    pub fn free_edges(&self) -> usize {
        self.total_edges - self.total_internal
    }

    /// The total number of edges between all of the points
    pub fn total_edges(&self) -> usize {
        self.total_edges
    }

    pub fn total_size(&self) -> u32 {
        self.size
    }

    pub fn iter(&self) -> SizedUnionFindIntoIter {
        SizedUnionFindIntoIter {
            size: self.total_size(),
            index: 0,
        }
    }
}

impl Index<Point> for SizedUnionFind {
    type Output = LinkSizeCompact;

    fn index(&self, index: Point) -> &Self::Output {
        &self.data[index.0 as usize]
    }
}

pub struct SizedUnionFindIntoIter {
    size: u32,
    index: u32,
}

impl Iterator for SizedUnionFindIntoIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let point = Point(self.index);
            self.index += 1;
            return Some(point);
        }

        None
    }
}

impl Distribution<Point> for SizedUnionFind {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Point {
        Point(self.vertex_distr.sample(rng))
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, SizedUnionFind};

    #[test]
    fn root() {
        let mut set = SizedUnionFind::new(10);
        assert_eq!(set.root(Point(2)), Point(2));
        assert_eq!(set.root(Point(3)), Point(3));
        assert!(!set.same_set(Point(3), Point(2)));
        set.link(Point(2), Point(3));
        assert_eq!(set.root(Point(2)), Point(3));
        assert_eq!(set.root(Point(3)), Point(3));
    }

    #[test]
    fn unite() {
        let mut set = SizedUnionFind::new(10);
        set.unite(Point(2), Point(3));
        set.unite(Point(3), Point(4));
        assert_eq!(set.root(Point(2)), Point(4));
        assert_eq!(set.root(Point(3)), Point(4));
        assert_eq!(set.root(Point(4)), Point(4));
    }

    #[test]
    fn simple() {
        let mut set = SizedUnionFind::new(10);

        assert_eq!(set.same_set(Point(1), Point(2)), false);
        set.unite(Point(1), Point(2));
        assert_eq!(set.same_set(Point(1), Point(2)), true);

        set.unite(Point(1), Point(7));
        set.unite(Point(2), Point(3));
        set.unite(Point(4), Point(5));
        assert_eq!(set.same_set(Point(4), Point(1)), false);

        set.unite(Point(4), Point(2));
        assert_eq!(set.same_set(Point(4), Point(1)), true);

        assert_eq!(set.linked_edges(), 15);
        assert_eq!(set.free_edges(), 30);
    }
}

#[cfg(all(test, feature = "benchmark"))]
mod benchmarks {
    extern crate test;
    use super::SizedUnionFind;
    use rand::{thread_rng, Rng};
    use test::{black_box, Bencher};

    const NUM_POINTS: u32 = 100_000;

    fn generate(ratio: f64) -> SizedUnionFind {
        let mut set = SizedUnionFind::new(NUM_POINTS);
        let mut rng = thread_rng();

        // Randomly unite `unite` edges
        for _ in 0..(NUM_POINTS as f64 * ratio) as usize {
            set.unite(rng.sample(&set), rng.sample(&set));
        }

        set
    }

    fn sets() -> Vec<SizedUnionFind> {
        vec![generate(0.1), generate(0.5), generate(0.8)]
    }

    #[bench]
    fn unite(b: &mut Bencher) {
        let mut rng = thread_rng();
        for mut set in sets() {
            b.iter(|| {
                black_box(set.unite(rng.sample(&set), rng.sample(&set)));
            });
        }
    }

    #[bench]
    fn same_set(b: &mut Bencher) {
        let mut rng = thread_rng();
        for mut set in sets() {
            b.iter(|| {
                black_box(set.same_set(rng.sample(&set), rng.sample(&set)));
            });
        }
    }

    #[bench]
    fn root(b: &mut Bencher) {
        let mut rng = thread_rng();
        for mut set in sets() {
            b.iter(|| {
                black_box(set.root(rng.sample(&set)));
            });
        }
    }

    #[bench]
    fn same_set_unite(b: &mut Bencher) {
        let mut rng = thread_rng();
        for mut set in sets() {
            b.iter(|| {
                let u = rng.sample(&set);
                let v = rng.sample(&set);
                if !set.same_set(u, v) {
                    black_box(set.unite(u, v));
                }
            });
        }
    }
}
