use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Exp1};

use crate::graph::Edge;

pub mod rem_union_find;
pub mod sized_rem_union_find;

// Generic union find struct that can be used in Kruskal's algorithm
pub trait KruskalUnionFind {
    fn new(size: u32) -> Self;
    fn unite(&mut self, u: u32, v: u32) -> bool;
}

pub trait SizedKruskalUnionFind {
    fn new(size: u32) -> Self;
    fn unite(&mut self, u: u32, v: u32) -> Option<(u32, u32)>;
}

// Some generator of sorted edges for use in Kruskal's algorithm
pub trait KruskalEdgeGenerator: IntoIterator<Item = Edge> {
    fn size(&self) -> u32;
}

pub fn mst_total_length<U: KruskalUnionFind>(edges: impl KruskalEdgeGenerator) -> f64 {
    let mut set = U::new(edges.size());
    let mut total_count = edges.size() - 1;
    let mut total_weight = 0.0;

    for edge in edges.into_iter() {
        if set.unite(edge.u, edge.v) {
            // Add edge to total graph
            total_weight += edge.w;
            total_count -= 1;
        }

        // If we have a tree, we're done
        if total_count == 0 {
            break;
        }
    }

    total_weight
}

pub fn mst_total_length_fat_component<U: SizedKruskalUnionFind>(size: u32) -> f64 {
    let mut set = U::new(size);
    let mut total_count = size - 1;
    let mut total_weight = 0.0;
    let mut total_internal = (size as usize - 1) * size as usize / 2;

    let mut weight = 0.0;

    let mut rng = thread_rng();

    loop {
        let mut u = rng.gen_range(0..size);
        let mut v = rng.gen_range(0..size);
        let mut sizes = set.unite(u, v);
        while sizes.is_none() {
            u = rng.gen_range(0..size);
            v = rng.gen_range(0..size);
            sizes = set.unite(u, v);
        }

        let sizes = sizes.unwrap();
        // TODO: switch to more accurate
        weight += rng.sample::<f64, _>(Exp1) / (total_internal as f64);
        total_internal -= sizes.0 as usize * sizes.1 as usize;

        total_weight += weight;
        total_count -= 1;

        // If we have a tree, we're done
        if total_count == 0 {
            break;
        }
    }

    total_weight
}
