use smallvec::SmallVec;

use super::{morton::Morton, point::Point};

pub struct Cell<const D: usize> {
    points: SmallVec<[Point<D>; 4]>,
}

impl<const D: usize> Cell<D> {
    fn new() -> Self {
        Self {
            points: SmallVec::new(),
        }
    }
}

pub struct SpatialVec<const D: usize> {
    cells: Vec<Cell<D>>,
}

impl<const D: usize> SpatialVec<D>
where
    Point<D>: Morton,
{
    pub fn new(points: impl Iterator<Item = Point<D>> + ExactSizeIterator) -> Self {
        // Calculate optimal resolution (TODO: fix)
        let size = points.len() as u32;
        let resolution = (u32::BITS - size.leading_zeros()) / D as u32;
        let cell_count: u64 = 1 << (resolution * D as u32);

        let mut cells: Vec<Cell<D>> = (0..cell_count).map(|_| Cell::new()).collect();
        for point in points {
            let morton = point.morton_encode();
            cells[morton as usize].points.push(point);
        }

        Self { cells }
    }
}
