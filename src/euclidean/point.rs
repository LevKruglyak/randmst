use super::morton::{morton_encode_2, morton_encode_3, morton_encode_4, Morton};
use rand_distr::Distribution;

pub struct Point<const D: usize>([u64; D]);

impl<const D: usize> Point<D> {
    fn distance2_fixed(&self, point: &Point<D>) -> u64 {
        let mut sum = 0_u128;

        for i in 0..D {
            let delt = self.0[i].wrapping_sub(point.0[i]) as u128;
            sum = sum.wrapping_add(delt.wrapping_mul(delt));
        }

        (sum >> u64::BITS) as u64
    }

    fn round_to_u32(&self) -> [u32; D] {
        // GODBOLT: bit shift gets optimized away, SIMD in D=4
        self.0.map(|x| (x >> (u64::BITS - u32::BITS)) as u32)
    }

    fn round_to(&self, bits: u32) -> [u32; D] {
        self.0.map(|x| (x >> (u64::BITS - bits)) as u32)
    }
}

impl Morton for Point<2> {
    fn morton_encode(&self, resolution: u32) -> usize {
        morton_encode_2(self.round_to(resolution))
    }
}

impl Morton for Point<3> {
    fn morton_encode(&self, resolution: u32) -> usize {
        morton_encode_3(self.round_to(resolution))
    }
}

impl Morton for Point<4> {
    fn morton_encode(&self, resolution: u32) -> usize {
        morton_encode_4(self.round_to(resolution))
    }
}

fn mul_fixed(x: u64, y: u64) -> u64 {
    // GODBOLT: completely optimized to a `mul` and `mov`
    ((x as u128) * (y as u128) >> u64::BITS) as u64
}

fn add_fixed(x: u64, y: u64) -> u64 {
    x + y
}

pub fn fixed_to_float(x: u64) -> f64 {
    const MANTISSA_BITS: usize = 52;
    const FIXED_UNIT: u64 = 0x0100_0000_0000_0000;

    (x >> (u64::BITS - 52)) as f64 / (FIXED_UNIT as f64)
}

pub struct Hypercube<const D: usize>;

macro_rules! hypercube_impl {
    ($D:expr) => {
        impl Distribution<Point<$D>> for Hypercube<$D> {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Point<$D> {
                Point(rng.gen::<[u64; $D]>())
            }
        }
    };
}

hypercube_impl!(2);
hypercube_impl!(3);
hypercube_impl!(4);

#[cfg(all(test, feature = "benchmark"))]
mod tests {
    use rand::{thread_rng, Rng};
    use test::{black_box, Bencher};

    use super::{fixed_to_float, Hypercube, Point};

    extern crate test;

    const D: usize = 4;

    #[bench]
    fn distance2_fixed(b: &mut Bencher) {
        let points: Vec<Point<D>> = thread_rng().sample_iter(Hypercube::<D>).take(2).collect();

        b.iter(|| black_box(points[0].distance2_fixed(&points[1])))
    }

    #[bench]
    fn distance2_float(b: &mut Bencher) {
        let points: ([f64; D], [f64; D]) = thread_rng().gen();

        b.iter(|| {
            black_box({
                let mut dist2 = 0.0;
                let point0 = black_box(points.0);
                let point1 = black_box(points.1);

                for i in 0..D {
                    dist2 += (point0[i] - point1[i]) * (point0[i] - point1[i]);
                }

                black_box(dist2)
            });
        });
    }

    #[bench]
    fn convert_back_to_float(b: &mut Bencher) {
        let point: u64 = thread_rng().gen();

        b.iter(|| {
            black_box(fixed_to_float(point));
        });
    }
}
