use rand::{Rng, RngCore};
use rand_distr::Distribution;

use self::{
    morton::Morton,
    point::{Hypercube, Point},
    spatial::SpatialVec,
};

mod morton;
mod point;
mod spatial;

pub fn mst<const D: usize>(size: u32, mut rng: impl RngCore) -> f64
where
    Hypercube<D>: Distribution<Point<D>>,
    Point<D>: Morton,
{
    let spatial = SpatialVec::new((0..size).map(|_| rng.sample(Hypercube::<D>)));

    0.0
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};
    use test::{black_box, Bencher};

    use super::{
        morton::Morton,
        point::{Hypercube, Point},
    };

    extern crate test;

    const D: usize = 4;
    const SIZE: usize = 262144;

    #[bench]
    fn generate(b: &mut Bencher) {
        b.iter(|| {
            let points: Vec<Point<D>> = black_box(
                thread_rng()
                    .sample_iter(Hypercube::<D>)
                    .take(SIZE)
                    .collect(),
            );
            points
        })
    }

    #[bench]
    fn morton_code(b: &mut Bencher) {
        let points: Vec<Point<D>> = thread_rng()
            .sample_iter(Hypercube::<D>)
            .take(SIZE)
            .collect();

        b.iter(|| {
            let morton: Vec<u32> = black_box(points.iter().map(|x| x.morton_encode(32)).collect());
            morton
        });
    }
}
