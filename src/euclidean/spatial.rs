use smallvec::SmallVec;

use super::{aabb::AABB, morton::Morton, point::Point};

/// A set of points representing the recursion level
/// at which it becomes more efficient to simply do
/// a direct MST algorithm on the complete graph
pub struct BaseCell<const D: usize> {
    points: SmallVec<[Point<D>; 8]>,
    bounds: AABB<D>,
}

impl<const D: usize> BaseCell<D> {
    fn new() -> Self {
        Self {
            points: SmallVec::new(),
            bounds: AABB::default(),
        }
    }

    fn push(&mut self, point: Point<D>) {
        self.points.push(point);
        self.bounds = AABB::expand(self.bounds.clone(), point);
    }
}

pub struct SpatialVec<const D: usize> {
    cells: Vec<BaseCell<D>>,
}

impl<const D: usize> SpatialVec<D>
where
    Point<D>: Morton,
{
    // Play with this to get best average
    const LEN_FACTOR: usize = 1;

    pub fn new(points: impl Iterator<Item = Point<D>> + ExactSizeIterator) -> Self {
        let mut cells = (points.len() / Self::LEN_FACTOR) as u32;
        let mut resolution = cells.ilog2() / D as u32;
        cells = 1 << (D as u32 * resolution);

        if points.len() < u32::BITS as usize * D {
            unimplemented!();
        }

        // println!("cells: {cells}");
        // println!("resolution: {resolution}");

        let mut cells: Vec<BaseCell<D>> = (0..cells).map(|_| BaseCell::<D>::new()).collect();

        for point in points {
            let index = point.morton_encode(resolution);
            cells[index].push(point);
        }

        let mut avg = 0_usize;
        let mut heap_count = 0_usize;
        for cell in &cells {
            if cell.points.len() != 0 {
                avg += cell.points.len();
            }

            if cell.points.len() > 8 {
                heap_count += 1;
            }
        }
        // println!("avg: {}", avg / cells.len());
        // println!("heap: {}", heap_count);

        Self { cells }
    }
}

#[cfg(test)]
mod tests {}
