use std::fmt::Display;

use super::point::Point;

#[derive(Clone, Debug)]
pub struct AABB<const D: usize> {
    min: Point<D>,
    max: Point<D>,
}

impl<const D: usize> Display for AABB<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("min: {}, ", self.min))?;
        f.write_fmt(format_args!("max: {},", self.max))?;

        Ok(())
    }
}

impl<const D: usize> Default for AABB<D> {
    fn default() -> Self {
        Self {
            min: Point::from([u64::MAX; D]),
            max: Point::from([u64::MIN; D]),
        }
    }
}

impl<const D: usize> AABB<D> {
    pub fn new(min: Point<D>, max: Point<D>) -> Self {
        Self { min, max }
    }

    /// Assumes point is within box
    pub fn dist2(&self, point: Point<D>) -> u64 {
        let dmin = point - self.min;
        let dmax = self.max - point;
        let delt = dmin.min(dmax);
        *delt.coords().iter().min().unwrap()
    }

    pub fn point(point: Point<D>) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn union(first: Self, second: Self) -> Self {
        Self {
            min: first.min.min(second.min),
            max: first.max.max(second.max),
        }
    }

    pub fn expand(first: Self, point: Point<D>) -> Self {
        Self {
            min: first.min.min(point),
            max: first.max.max(point),
        }
    }

    pub fn diag2(&self) -> u64 {
        self.min.distance2_fixed(&self.max)
    }
}
