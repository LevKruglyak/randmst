use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::Exp1;

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
    fn find(&self, u: u32) -> u32;
    fn size(&self, u: u32) -> u32;
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

struct FatComponentGenerator<U: SizedKruskalUnionFind> {
    rng: ThreadRng,
    total_edges: usize,
    total_internal: usize,
    size: u32,
    weight: f64,
    inv_min: f64,
    fat_component: Option<u32>,
    remainders: Vec<u32>,
    union_find: U,
}

#[derive(Debug)]
struct VertPair(u32, u32);

impl<U: SizedKruskalUnionFind> FatComponentGenerator<U> {
    fn new(size: u32) -> Self {
        Self {
            rng: thread_rng(),
            total_edges: (size as usize - 1) * size as usize / 2,
            total_internal: 0,
            size,
            weight: 0.0,
            inv_min: 1.0,
            fat_component: None,
            remainders: Vec::new(),
            union_find: U::new(size),
        }
    }

    fn sample_sparse(&mut self) -> VertPair {
        // Rejection sample until we get an acylcic addition
        loop {
            let u = self.rng.gen_range(0..self.size);
            let v = self.rng.gen_range(0..self.size);
            if let Some((s1, s2)) = self.union_find.unite(u, v) {
                self.total_internal += s1 as usize * s2 as usize;
                return VertPair(u, v);
            }
        }
    }

    fn sample_fat_component_vertex(&mut self, fat_component: u32) -> u32 {
        // Rejection sample until we hit a vertex in the fat component
        loop {
            let u = self.rng.gen_range(0..self.size);
            if self.union_find.find(u) == fat_component {
                return u;
            }
        }
    }

    fn sample_remainder_vertex(&mut self, fat_component: u32) -> u32 {
        loop {
            let index = self.rng.gen_range(0..self.remainders.len());
            let u = self.remainders[index];
            if self.union_find.find(u) != fat_component {
                return u;
            }
        }
    }

    fn sample_fat_component_edge(&mut self, fat_component: u32) -> VertPair {
        let fat_size = self.union_find.size(fat_component) as usize;
        let remainder_size = (self.size - fat_size as u32) as usize;
        let active_edges = self.total_edges - self.total_internal;

        if self
            .rng
            .gen_bool((fat_size * remainder_size) as f64 / active_edges as f64)
        {
            // generate a random edge between active and remainder.
            VertPair(
                self.sample_fat_component_vertex(fat_component),
                self.sample_remainder_vertex(fat_component),
            )
        } else {
            loop {
                let u = self.sample_remainder_vertex(fat_component);
                let v = self.sample_remainder_vertex(fat_component);
                if let Some((s1, s2)) = self.union_find.unite(u, v) {
                    self.total_internal += s1 as usize * s2 as usize;
                    return VertPair(u, v);
                }
            }
        }
    }

    fn find_fat_component(&mut self) {
        for v in 0..self.size {
            if self.union_find.size(v) * 2 >= self.size {
                let root = self.union_find.find(v);
                self.fat_component = Some(root);
                for w in 0..self.size {
                    if self.union_find.find(w) != root {
                        self.remainders.push(w);
                    }
                }
                return;
            }
        }
        unreachable!() // this means that the existence of a fat component wasn't checked first
    }

    fn check_and_update_remainders(&mut self) {
        match self.fat_component {
            Some(fat_component) => {
                if (self.size - self.union_find.size(fat_component)) * 2
                    < self.remainders.len() as u32
                {
                    let mut new_remainders = Vec::new();
                    let fat_root = self.union_find.find(fat_component);
                    for &v in &self.remainders {
                        if self.union_find.find(v) != fat_root {
                            new_remainders.push(v);
                        }
                    }
                    self.remainders = new_remainders;
                }
            }
            None => {}
        }
    }
}

impl<U: SizedKruskalUnionFind> Iterator for FatComponentGenerator<U> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let active_edges = self.total_edges - self.total_internal;
        if active_edges == 0 {
            return None;
        }

        let r: f64 = self.rng.gen();
        self.inv_min *= r.powf(1.0_f64 / active_edges as f64);
        self.weight += self.rng.sample::<f64, _>(Exp1) / active_edges as f64;
        self.check_and_update_remainders();

        if active_edges * 2 < self.total_edges && self.fat_component.is_none() {
            self.find_fat_component();
        }

        let edge = match self.fat_component {
            Some(component) => self.sample_fat_component_edge(component),
            None => self.sample_sparse(),
        };

        Some(1.0 - self.inv_min)
    }
}

pub fn mst_total_length_fat_component<U: SizedKruskalUnionFind>(size: u32) -> f64 {
    let gen = FatComponentGenerator::<U>::new(size);
    let mut total_count = size - 1;
    let mut total_weight = 0.0;

    for weight in gen {
        total_weight += weight;
        total_count -= 1;

        if total_count == 0 {
            break;
        }
    }

    total_weight
}
