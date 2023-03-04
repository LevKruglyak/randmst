use std::{cell::Cell, ops::Index};

use crate::euclidean::point::fixed_to_float;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Point(u32);

pub struct UnionFind {
    links: Vec<Cell<Point>>,
}

impl UnionFind {
    pub fn new(size: u32) -> Self {
        Self {
            links: (0..size).map(|x| Cell::new(Point(x))).collect(),
        }
    }

    pub fn unite(&self, mut u: Point, mut v: Point) -> bool {
        let mut x = self.root(u);
        let mut y = self.root(v);
        if x == y {
            return false;
        }

        if x < y {
            std::mem::swap(&mut x, &mut y);
        }

        self[y].set(x);
        true
    }

    pub fn root(&self, u: Point) -> Point {
        let mut root = u;
        if root == self[root].get() {
            return root;
        }

        let mut prev = &self[root];

        loop {
            let temp = self[root].get();
            if temp == self[temp].get() {
                return temp;
            }

            // Path splitting
            prev.set(self[temp].get());
            prev = &self[temp];
            root = temp;
        }
    }

    pub fn clear(&mut self) {
        // TODO: see if this is actually faster than
        // just reallocating everything
        let size = self.links.len() as u32;
        self.links
            .iter_mut()
            .zip((0..size))
            .map(|(x, i)| *x = Cell::new(Point(i)));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dist2Edge {
    pub u: u32,
    pub v: u32,
    pub dist2: u64,
}

// TODO: compactify, pack bit into dist field
pub enum MaybeEdge {
    Sure(Dist2Edge),
    Maybe(Dist2Edge),
}

pub fn kruskal(edges: &mut Vec<Dist2Edge>, size: usize) -> Vec<Dist2Edge> {
    if size == 0 {
        return vec![];
    }

    // Sort by weight
    edges.sort_by(|x, y| x.dist2.cmp(&y.dist2));

    let mut mst = Vec::with_capacity(size - 1);
    let mut counter = size - 1;
    let mut union = UnionFind::new(size as u32);

    for &mut edge in edges {
        if counter == 0 {
            break;
        }

        if union.unite(Point(edge.u), Point(edge.v)) {
            mst.push(edge);
            counter -= 1;
        }
    }

    assert_eq!(counter, 0);

    mst
}

impl Index<Point> for UnionFind {
    type Output = Cell<Point>;

    fn index(&self, index: Point) -> &Self::Output {
        &self.links[index.0 as usize]
    }
}
