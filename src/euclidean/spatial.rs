use rand::{thread_rng, Rng};
use smallvec::SmallVec;

use super::{
    aabb::AABB,
    kruskal::{kruskal, Dist2Edge, MaybeEdge},
    morton::Morton,
    point::{self, Point},
};

/// A set of points representing the recursion level
/// at which it becomes more efficient to simply do
/// a direct MST algorithm on the complete graph
#[derive(Debug)]
pub struct BaseCell<const D: usize> {
    points: SmallVec<[u32; 8]>,

    // TODO: is precalculating this bound better?
    bounds: AABB<D>,
}

impl<const D: usize> BaseCell<D> {
    fn new() -> Self {
        Self {
            points: SmallVec::new(),
            bounds: AABB::default(),
        }
    }

    fn push(&mut self, index: u32, point: Point<D>) {
        self.points.push(index);
        self.bounds = AABB::expand(self.bounds.clone(), point);
    }
}

pub enum SpatialSliceSplit<'a, const D: usize> {
    Split(SpatialSlice<'a, D>, SpatialSlice<'a, D>),
    Root(&'a mut BaseCell<D>, &'a [(Point<D>, u32)]),
}

/// Represents a `half slice` of the spatial vector
pub struct SpatialSlice<'a, const D: usize> {
    cells: &'a mut [BaseCell<D>],
    zord: &'a [(Point<D>, u32)],
}

impl<const D: usize> SpatialSlice<'_, D> {
    pub fn split<'a>(&'a mut self) -> SpatialSliceSplit<'a, D> {
        use SpatialSliceSplit::*;

        if self.cells.len() > 1 {
            // Shouldn't need to round up because slice length will always
            // be a power of 2, but just in case
            let (s1, s2) = self.cells.split_at_mut((self.cells.len() + 1) / 2);
            Split(
                SpatialSlice {
                    cells: s1,
                    zord: &self.zord[..],
                },
                SpatialSlice {
                    cells: s2,
                    zord: &self.zord[..],
                },
            )
        } else {
            Root(&mut self.cells[0], &self.zord[..])
        }
    }
}

/// A slice that has been recursively merged all the way down
pub struct MergedSlice<const D: usize> {
    // slice: SpatialSlice<'a, D>,
    partial_graph: Vec<MaybeEdge>,
    bounds: AABB<D>,
}

impl<const D: usize> MergedSlice<D> {
    /// Recursively merge a slice
    pub fn recursive_merge(mut slice: SpatialSlice<'_, D>) -> Self {
        use SpatialSliceSplit::*;

        match slice.split() {
            Split(s1, s2) => Self::merge(Self::recursive_merge(s1), Self::recursive_merge(s2)),
            Root(root, zord) => Self::kruskal_root(root, zord),
        }
    }

    pub fn merge(mut first: MergedSlice<D>, mut second: MergedSlice<D>) -> Self {
        first.partial_graph.append(&mut second.partial_graph);

        Self {
            bounds: AABB::union(first.bounds, second.bounds),
            partial_graph: first.partial_graph,
        }
    }

    pub fn kruskal_root(root: &mut BaseCell<D>, zord: &[(Point<D>, u32)]) -> Self {
        let mut edges: Vec<Dist2Edge> = Vec::new();
        for i in 0..root.points.len() {
            for j in 0..i {
                let u = root.points[i];
                let v = root.points[j];
                edges.push(Dist2Edge {
                    u: i as u32,
                    v: j as u32,
                    dist2: zord[u as usize].0.distance2_fixed(&zord[v as usize].0),
                })
            }
        }

        // First pass
        let first = kruskal(&mut edges, root.points.len(), |_| true);

        // Add boundary points
        let boundary = root.points.len() as u32;
        for (i, &point) in root.points.iter().enumerate() {
            edges.push(Dist2Edge {
                u: i as u32,
                v: boundary, // represents the boundary `node`
                dist2: root.bounds.dist2(zord[point as usize].0),
            })
        }

        // slightly larger union find set to account for the boundary `node`
        let second = kruskal(&mut edges, root.points.len() + 1, |edge| {
            (edge.u != boundary) & (edge.v != boundary)
        });

        // Merge the results of the Kruskal's
        let partial_graph: Vec<MaybeEdge> = first
            .into_iter()
            .map(|x| {
                if let Ok(_) = second.binary_search_by(|y| y.dist2.cmp(&x.dist2)) {
                    MaybeEdge::Sure(x)
                } else {
                    MaybeEdge::Maybe(x)
                }
            })
            .collect();

        Self {
            bounds: root.bounds.clone(),
            // TODO: remember to map the points back into the correct representation!!!
            partial_graph,
        }
    }
}

pub struct SpatialVec<const D: usize> {
    cells: Vec<BaseCell<D>>,

    // Points sorted by Z-order
    zord: Vec<(Point<D>, u32)>,
}

impl<const D: usize> SpatialVec<D>
where
    Point<D>: Morton,
{
    // Play with this to get best average
    const LEN_FACTOR: usize = match D {
        _ => 1,
    };

    pub fn new(points: impl Iterator<Item = Point<D>> + ExactSizeIterator) -> Self {
        let mut cells = (points.len() * Self::LEN_FACTOR) as u32;
        let mut resolution = cells.ilog2() / D as u32;
        cells = 1 << (D as u32 * resolution);

        if points.len() < u32::BITS as usize * D {
            unimplemented!();
        }

        println!("cells: {cells}");
        println!("resolution: {resolution}");

        let mut cells: Vec<BaseCell<D>> = (0..cells).map(|_| BaseCell::<D>::new()).collect();
        let mut zord: Vec<(Point<D>, u32)> = points
            .map(|p| (p, p.morton_encode(resolution) as u32))
            .collect();

        // Z-ordering of points in global array
        zord.sort_by(|x, y| x.1.cmp(&y.1));

        // Cache friendly insertion (according to the Z-ordering)
        for (i, &(point, z)) in zord.iter().enumerate() {
            cells[z as usize].push(i as u32, point);
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
        println!("avg: {}", avg / cells.len());
        println!("heap: {}", heap_count);

        Self { cells, zord }
    }

    pub fn as_slice(&mut self) -> SpatialSlice<D> {
        SpatialSlice {
            cells: self.cells.as_mut_slice(),
            zord: &self.zord[..],
        }
    }
}

pub fn recurse_test<const D: usize>(mut slice: SpatialSlice<D>) -> bool
where
    Point<D>: Morton,
{
    let merge = MergedSlice::recursive_merge(slice);
    println!("world bound: {}", merge.bounds);
    println!("partial_graph: {}", merge.partial_graph.len());

    true
    // match slice.split() {
    //     SpatialSliceSplit::Split(a, b) => {
    //         return recurse_test(a) || recurse_test(b);
    //     }
    //     SpatialSliceSplit::Root(root) => {
    //         let dist = point::fixed_to_float(root.bounds.diag2());
    //         if root.points.len() != 0 {
    //             println!("{}, {dist} size {}", root.bounds, root.points.len());
    //         }
    //
    //         return root.points.len() != 0;
    //     }
    // }
}

#[cfg(test)]
mod tests {}
