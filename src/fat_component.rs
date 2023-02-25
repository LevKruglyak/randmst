use rand::{Rng, RngCore};
use rand_distr::Exp1;

pub fn mst(size: u32, rng: impl RngCore) -> f64 {
    let gen = FatComponentGenerator::new(rng, size);
    gen.sum()
}

struct FatComponent {
    root: Point,
    size: u32,
    // All of the points not in the fat component
    remainders: Vec<Point>,
}

impl FatComponent {
    fn new(root: Point, size: u32) -> Self {
        Self {
            root,
            size,
            remainders: Vec::new(),
        }
    }

    fn root(&self) -> Point {
        self.root
    }

    fn update(&mut self, root: Point, size: u32) {
        self.root = root;
        self.size = size;
    }

    fn insert_remainder(&mut self, remainder: Point) {
        self.remainders.push(remainder);
    }

    fn remainders(&self) -> u32 {
        self.remainders.len() as u32
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn filter(&mut self, mut filter: impl FnMut(&Point) -> bool) {
        self.remainders = self
            .remainders
            .clone()
            .into_iter()
            .filter(|x| filter(x))
            .collect();
    }
}

/// A point in the union find
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord)]
struct Point(u32);

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

struct FatComponentGenerator<R: RngCore> {
    weight: f64,
    set: FatComponentUnionFind,
    rng: R,
}

impl<R: RngCore> FatComponentGenerator<R> {
    fn new(rng: R, size: u32) -> Self {
        Self {
            weight: 0.0,
            set: FatComponentUnionFind::new(size),
            rng,
        }
    }
}

impl<R: RngCore> Iterator for FatComponentGenerator<R> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let active_edges = self.set.active_edges();
        if active_edges == 0 {
            return None;
        }

        self.weight += self.rng.sample::<f64, _>(Exp1) / active_edges as f64;
        self.set.sample(&mut self.rng);
        Some(self.weight)
    }
}

/// A grow-only union find data structure that
/// also stores a `fat component` when a singular
/// set grows bigger than a given threshold
struct FatComponentUnionFind {
    parents: Vec<Point>,
    sizes: Vec<u32>,
    fat_component: Option<FatComponent>,
    /// Number of total internal edges between the points
    total_internal: usize,
    total_edges: usize,
    size: u32,
}

#[allow(unused)]
impl FatComponentUnionFind {
    pub fn new(size: u32) -> Self {
        Self {
            parents: (0..size).into_iter().map(|x| Point(x)).collect(),
            sizes: (0..size).into_iter().map(|_| 1).collect(),
            fat_component: None,
            total_internal: 0,
            total_edges: (size as usize) * (size as usize - 1) / 2,
            size,
        }
    }

    // Are these points members of the same set?
    pub fn same_set(&self, first: Point, second: Point) -> bool {
        // TODO: add speedup for fat component
        self.root(first) == self.root(second)
    }

    // Traverse up the tree of the current component to get the root
    pub fn root(&self, mut point: Point) -> Point {
        while !self.is_root(point) {
            point = self.parent(point);
        }

        point
    }

    // Merge the sets of `first` and `second` using Rem's algorithm
    pub fn unite(&mut self, mut first: Point, mut second: Point) -> bool {
        while self.parent(first) != self.parent(second) {
            // Make sure we're oriented propertly to keep
            // the root at the end of the array
            if self.parent(first) > self.parent(second) {
                std::mem::swap(&mut first, &mut second);
            }

            // If we've reached a root, we're done
            if let Some(join_size) = self.point_size(first) {
                // Link `first` to parent of `second`
                *self.parent_mut(first) = self.parent(second);
                let (size, root) = self.add_size_to_root(second, join_size);

                // Update the internal edges counter
                self.total_internal += size as usize * join_size as usize;

                // If fat component exists, make sure it is up to date
                if size + join_size > self.size / 2 {
                    // Borrow checker trickery
                    match self.fat_component.as_mut() {
                        None => {
                            println!("found fat component!");
                            let mut fat_component = FatComponent::new(root, size + join_size);

                            // Add all of the remainders
                            for point in 0..self.size {
                                fat_component.insert_remainder(Point(point));
                            }

                            self.fat_component = Some(fat_component);
                        }
                        Some(fat_component) => {
                            // Update the location of the fat component
                            fat_component.update(root, size + join_size);

                            // Update all of the remainders (TODO: fix borrow checker trickery)
                            if (self.size - fat_component.size()) * 2 < fat_component.remainders() {
                                let mut fat_component = self.fat_component.take().unwrap();
                                let root = fat_component.root();
                                fat_component.filter(|point| self.root(*point) != root);
                                self.fat_component = Some(fat_component);
                            }
                        }
                    };
                }

                return true;
            }

            // Path splitting step
            let temp = self.parent(first);
            *self.parent_mut(first) = self.parent(second);
            first = temp;
        }

        false
    }

    // We don't need to return anything since edge weights are independent
    pub fn sample(&mut self, rng: &mut impl RngCore) {
        let active_edges = self.active_edges();
        // TODO: fix borrow checker shenanigans
        if let Some(fat_component) = self.fat_component.take() {
            let fat_size = fat_component.size() as usize;
            let remainder_size = (self.size - fat_size as u32) as usize;

            if rng.gen_bool((fat_size * remainder_size) as f64 / active_edges as f64) {
                // generate a random edge between active and remainder.
                let edge = (
                    self.sample_component(fat_component.root(), rng),
                    self.sample_remainder_vector(
                        fat_component.root(),
                        &fat_component.remainders,
                        rng,
                    ),
                );
                // assert!(self.unite(edge.0, edge.1));
            } else {
                let edge = self.sample_remainder_vector_pair(
                    fat_component.root(),
                    &fat_component.remainders,
                    rng,
                );
                // assert!(self.unite(edge.0, edge.1));
            }

            self.fat_component = Some(fat_component);
        } else {
            let edge = self.sample_sparse(rng);
            // assert!(self.unite(edge.0, edge.1));
        };
    }

    fn sample_component(&mut self, component: Point, rng: &mut impl RngCore) -> Point {
        // Rejection sample until we hit a vertex in the given component
        loop {
            let point = Point(rng.gen_range(0..self.size));
            if self.root(point) == component {
                return point;
            }
        }
    }

    fn sample_remainder_vector(
        &mut self,
        component: Point,
        remainders: &Vec<Point>,
        rng: &mut impl RngCore,
    ) -> Point {
        loop {
            let index = rng.gen_range(0..remainders.len());
            let point = remainders[index];
            if self.root(point) != point {
                return point;
            }
        }
    }

    fn sample_remainder_vector_pair(
        &mut self,
        component: Point,
        remainders: &Vec<Point>,
        rng: &mut impl RngCore,
    ) -> (Point, Point) {
        loop {
            let first = self.sample_remainder_vector(component, remainders, rng);
            let second = self.sample_remainder_vector(component, remainders, rng);
            if !self.same_set(first, second) {
                return (first, second);
            }
        }
    }

    pub fn sample_sparse(&self, rng: &mut impl RngCore) -> (Point, Point) {
        // Rejection sample until we get an acylcic addition
        loop {
            let first = Point(rng.gen_range(0..self.size));
            let second = Point(rng.gen_range(0..self.size));
            if !self.same_set(first, second) {
                return (first, second);
            }
        }
    }

    /// Returns the number of total internal edges
    pub fn total_internal_edges(&self) -> usize {
        self.total_internal
    }

    pub fn active_edges(&self) -> usize {
        self.total_edges - self.total_internal
    }

    pub fn fat_component(&self) -> Option<&FatComponent> {
        self.fat_component.as_ref()
    }

    #[inline]
    fn parent(&self, point: Point) -> Point {
        self.parents[point.0 as usize]
    }

    #[inline]
    fn parent_mut(&mut self, point: Point) -> &mut Point {
        &mut self.parents[point.0 as usize]
    }

    #[inline]
    fn is_root(&self, point: Point) -> bool {
        self.parent(point) == point
    }

    // If we're at a root, return the size, otherwise
    // return `None`
    #[inline]
    fn point_size(&self, point: Point) -> Option<u32> {
        if self.is_root(point) {
            return Some(self.sizes[point.0 as usize]);
        }

        None
    }

    // Traverse to the root and return the size
    fn root_size(&self, point: Point) -> u32 {
        self.sizes[self.root(point).0 as usize]
    }

    // Add a size to the given set, returning the original size
    fn add_size_to_root(&mut self, point: Point, size: u32) -> (u32, Point) {
        let root = self.root(point);
        let orig_size = self.sizes[root.0 as usize];
        self.sizes[root.0 as usize] += size;
        (orig_size, root)
    }
}

#[cfg(test)]
mod test {
    use crate::fat_component::Point;

    use super::FatComponentUnionFind;

    #[test]
    fn simple() {
        let fc = FatComponentUnionFind::new(100);
        assert!(fc.is_root(Point(10)));
        assert_eq!(fc.parent(Point(10)), Point(10));
        assert_eq!(fc.root_size(Point(10)), 1);
        assert_eq!(fc.point_size(Point(10)), Some(1));
    }
}
