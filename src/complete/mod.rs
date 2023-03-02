use rand::RngCore;

use crate::Edge;

use self::sampler::FatComponentSampler;

mod sampler;
pub mod union_find;

pub fn mst<R: RngCore>(size: u32, rng: R) -> Vec<Edge> {
    FatComponentSampler::<R>::new(rng, size).collect()
}

#[cfg(all(test, feature = "benchmark"))]
mod benchmarks {
    extern crate test;
    use super::mst;
    use rand::thread_rng;
    use test::{black_box, Bencher};

    /// Speed test from class
    #[bench]
    fn main(b: &mut Bencher) {
        b.iter(|| black_box(mst(262_144, thread_rng())));
    }
}
