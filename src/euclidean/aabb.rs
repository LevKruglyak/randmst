use super::point::Point;

#[derive(Default, Clone)]
pub struct AABB<const D: usize> {
    min: Point<D>,
    max: Point<D>,
}

impl<const D: usize> AABB<D> {
    pub fn new(min: Point<D>, max: Point<D>) -> Self {
        Self { min, max }
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
}
