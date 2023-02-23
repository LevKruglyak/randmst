use std::ops::{Index, IndexMut};

use super::{KruskalUnionFind, SizedKruskalUnionFind};

pub struct SizedRemUnionFind {
    parents: Vec<u32>,
    sizes: Vec<u32>,
}

const SENTINEL: u32 = 1 << 31;
const SENTINEL_MASK: u32 = !(1 << 31);

impl SizedRemUnionFind {
    fn root() -> u32 {
        1 | SENTINEL
    }

    fn to_parent(&self, u: u32) -> u32 {
        match u & SENTINEL {
            0 => self[u],
            _ => u & SENTINEL_MASK,
        }
    }
}

impl SizedKruskalUnionFind for SizedRemUnionFind {
    fn new(size: u32) -> Self {
        assert_eq!(size & SENTINEL, 0);

        Self {
            parents: (0..size).into_iter().map(|x| x).collect(),
            sizes: (0..size).into_iter().map(|_| 1).collect(),
        }
    }

    fn unite(&mut self, mut u: u32, mut v: u32) -> Option<(u32, u32)> {
        while self[u] != self[v] {
            // Make sure we're oriented properly to keep root nodes
            // at the end of the array
            if self[u] > self[v] {
                std::mem::swap(&mut u, &mut v);
            }

            // If we've reached a root, we're done
            if u == self[u] {
                let joning_size = self.sizes[u as usize];
                self[u] = self[v];
                // Want to find size of `self[v]` component
                while v != self[v] {
                    v = self[v];
                }

                let size = self.sizes[v as usize];
                self.sizes[v as usize] += joning_size;
                return Some((joning_size, size));
            }

            let z = self[u];
            self[u] = self[v];
            u = z;
        }

        None
    }
}

impl Index<u32> for SizedRemUnionFind {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        &self.parents[index as usize]
    }
}

impl IndexMut<u32> for SizedRemUnionFind {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.parents[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::kruskal::SizedKruskalUnionFind;

    use super::SizedRemUnionFind;

    // #[test]
    // fn startup() {
    //     let s = SizedRemUnionFind::new(100);
    //
    //     assert_eq!(s.root_size(10), Some(1));
    //     assert_eq!(s.root_size(87), Some(1));
    //     assert_eq!(s.root_size(33), Some(1));
    // }
    //
    // #[test]
    // fn parents() {
    //     let mut s = SizedRemUnionFind::new(100);
    //     assert_eq!(s.parent(10), 10);
    //     s.parents[11] = s.parent(10);
    //     assert_eq!(s.parent(11), 10);
    // }
    //
    // #[test]
    // fn simple_unite() {
    //     let mut s = SizedRemUnionFind::new(100);
    //     let sizes = s.unite(10, 87);
    //     assert_eq!(sizes, Some((1, 1)));
    //     assert_eq!(s.root_size(87), Some(2));
    //     assert_eq!(s.root_size(10), None);
    //
    //     let sizes = s.unite(20, 87);
    //     assert_eq!(sizes, Some((1, 2)));
    //     assert_eq!(s.root_size(87), Some(3));
    //     assert_eq!(s.root_size(20), None);
    // }
}
