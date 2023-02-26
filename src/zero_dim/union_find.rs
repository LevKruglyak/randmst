use rand_distr::{Distribution, Uniform};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Point(u32);

impl Point {
    fn root(id: u32) -> Self {
        Self(id)
    }
}

pub struct RemUnionFind {
    /// Sends each Point (as a usize) to its Parent. (roots are self-parent)
    parents: Vec<Point>,
    sizes: Vec<u32>,

    // Metadata to keep track of sizes for sampling purposes
    size: u32,
    total_edges: usize,
    total_internal: usize,

    // Marginal speedup when storing distribution object
    vertex_distr: Uniform<u32>,
}

impl RemUnionFind {
    pub fn new(size: u32) -> Self {
        Self {
            parents: (0..size).into_iter().map(Point::root).collect(),
            sizes: (0..size).into_iter().map(|_| 1).collect(),
            size,
            total_edges: (size as usize) * (size as usize - 1) / 2,
            total_internal: 0,
            vertex_distr: Uniform::new(0, size),
        }
    }

    /// Unites the two sets, returns the size and root of the united
    /// component. Returns `None` if these two points are in the same component
    pub fn unite(&mut self, mut u: Point, mut v: Point) -> Option<(u32, Point)> {
        while self.parent(u) != self.parent(v) {
            // Make sure we're oriented properly to keep root nodes
            // at the end of the array
            if self.parent(u) > self.parent(v) {
                std::mem::swap(&mut u, &mut v);
            }

            // If we've reached a root, we're done
            if let Some(join_size) = self.root_size(u) {
                self.link(u, self.parent(v));
                let (size, root) = self.add_size_to_root(v, join_size);

                // Update total internal edges
                self.total_internal += size as usize * join_size as usize;

                return Some((size, root));
            }

            let temp = self.parent(u);
            self.link(u, self.parent(v));
            u = temp;
        }

        // This means that these points were already part of the same component
        None
    }

    /// Are these two points in the same set?
    pub fn same_set(&mut self, mut u: Point, mut v: Point) -> bool {
        // ~ 20% speed improvement compared to:
        // self.root(u) == self.root(v)

        let u_orig = u;
        let v_orig = v;

        while !self.is_root(u) | !self.is_root(v) {
            u = self.parent(u);
            v = self.parent(v);
        }

        // Path splitting step
        self.link(u_orig, u);
        self.link(v_orig, v);

        u == v
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

    pub fn points(&self) -> u32 {
        self.size
    }

    pub fn points_iter(&self) -> RemUnionFindIntoIter {
        RemUnionFindIntoIter {
            size: self.points(),
            index: 0,
        }
    }

    /// Traverse to the root, splitting paths on the way
    pub fn root(&mut self, u: Point) -> Point {
        let mut root = u;

        while !self.is_root(root) {
            root = self.parent(root);
        }

        // connect `u` to its root
        self.link(u, root);

        root
    }

    pub fn size(&mut self, u: Point) -> u32 {
        let root = self.root(u);
        self.sizes[root.0 as usize]
    }

    /// If `u` is a root, will return `Some(size)`, otherwise `None`
    fn root_size(&self, u: Point) -> Option<u32> {
        if self.is_root(u) {
            Some(self.sizes[u.0 as usize])
        } else {
            None
        }
    }

    // Add a size to the given set, return the original size
    fn add_size_to_root(&mut self, point: Point, size: u32) -> (u32, Point) {
        let root = self.root(point);
        let original_size = self.sizes[root.0 as usize];
        self.sizes[root.0 as usize] += size;
        (original_size, root)
    }

    fn link(&mut self, src: Point, dst: Point) {
        self.parents[src.0 as usize] = dst
    }

    fn parent(&self, u: Point) -> Point {
        self.parents[u.0 as usize]
    }

    fn is_root(&self, u: Point) -> bool {
        self.parent(u) == u
    }
}

pub struct RemUnionFindIntoIter {
    size: u32,
    index: u32,
}

impl Iterator for RemUnionFindIntoIter {
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

impl Distribution<Point> for RemUnionFind {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Point {
        Point(self.vertex_distr.sample(rng))
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, RemUnionFind};

    #[test]
    fn root() {
        let mut set = RemUnionFind::new(10);
        assert!(set.is_root(Point(2)));
        assert!(set.is_root(Point(3)));
        set.unite(Point(2), Point(3));
        assert!(set.is_root(Point(3)));
    }

    #[test]
    fn simple() {
        let mut set = RemUnionFind::new(10);

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

#[cfg(test)]
mod benchmarks {
    extern crate test;
    use super::RemUnionFind;
    use rand::{thread_rng, Rng};
    use test::{black_box, Bencher};

    const NUM_POINTS: u32 = 100_000;

    fn generate(ratio: f64) -> RemUnionFind {
        let mut set = RemUnionFind::new(NUM_POINTS);
        let mut rng = thread_rng();

        // Randomly unite `unite` edges
        for _ in 0..(NUM_POINTS as f64 * ratio) as usize {
            set.unite(rng.sample(&set), rng.sample(&set));
        }

        set
    }

    fn sets() -> Vec<RemUnionFind> {
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
