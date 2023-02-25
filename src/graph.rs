use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{Distribution, Exp, Uniform};

use crate::kruskal::{rem_union_find::RemUnionFind, KruskalEdgeGenerator, KruskalUnionFind};

pub struct Edge {
    pub u: u32,
    pub v: u32,
    pub w: f64,
}

// An edge generator for the random zero dimensional case which
// lazily constructs the graph using an `approximate` exponential
// distribution. Will not be pratically perfect for N > 2^16
pub struct FastZeroDimEdgeGenerator {
    size: u32,
    total_weight: f64,
    total_count: u32,
    weight_distr: Exp<f64>,
    vertex_distr: Uniform<u32>,
    rng: ThreadRng,
}

impl FastZeroDimEdgeGenerator {
    pub fn new(size: u32) -> Self {
        let size_f = size as f64;
        Self {
            size,
            total_weight: 0.0,
            total_count: 0,
            weight_distr: Exp::new(size_f * size_f / 2.0)
                .expect("error creating exponential distribution"),
            vertex_distr: Uniform::new(0, size),
            rng: thread_rng(),
        }
    }
}

impl KruskalEdgeGenerator for FastZeroDimEdgeGenerator {
    fn size(&self) -> u32 {
        self.size
    }
}

impl Iterator for FastZeroDimEdgeGenerator {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        if self.total_count < self.size {
            self.total_weight += self.weight_distr.sample(&mut self.rng);
            let u = self.vertex_distr.sample(&mut self.rng);
            let v = self.vertex_distr.sample(&mut self.rng);

            return Some(Edge {
                u,
                v,
                w: self.total_weight,
            });
        }

        None
    }
}
