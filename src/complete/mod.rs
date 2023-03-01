use rand::RngCore;

use self::sampler::FatComponentSampler;

mod sampler;
pub mod union_find;

pub fn mst(size: u32, rng: impl RngCore) -> f64 {
    let mut sampler = FatComponentSampler::new(rng, size);
    let mut total_weight = 0.0;

    while let Some(weight) = sampler.sample() {
        total_weight += weight;
    }

    total_weight
}

// #[cfg(test)]
// mod benchmarks {
//     extern crate test;
//     use super::mst;
//     use rand::thread_rng;
//     use test::{black_box, Bencher};
//
//     /// Speed test from class
//     #[bench]
//     fn main(b: &mut Bencher) {
//         b.iter(|| black_box(mst(262_144, thread_rng())));
//     }
// }
