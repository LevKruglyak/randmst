use rand::{thread_rng, Rng};
use rand_distr::Exp1;

use self::{sampler::FatComponentSampler, union_find::RemUnionFind};

// use self::{sampler::FatComponentSampler, union_find::RemUnionFind};

mod sampler;
mod union_find;

pub fn mst(size: u32) -> f64 {
    let mut sampler = FatComponentSampler::new(thread_rng(), size);
    let mut total_weight = 0.0;
    let mut rng = thread_rng();

    while let Some(weight) = sampler.sample() {
        total_weight += weight;
    }

    total_weight
}
