use std::ops::{Index, IndexMut};

use super::KruskalUnionFind;

pub struct RemUnionFind {
    parents: Vec<u32>,
}

impl KruskalUnionFind for RemUnionFind {
    fn new(size: u32) -> Self {
        Self {
            parents: (0..size).into_iter().collect(),
        }
    }

    fn unite(&mut self, mut u: u32, mut v: u32) -> bool {
        while self[u] != self[v] {
            // Make sure we're oriented properly to keep root nodes
            // at the end of the array
            if self[u] > self[v] {
                std::mem::swap(&mut u, &mut v);
            }

            // If we've reached a root, we're done
            if u == self[u] {
                self[u] = self[v];
                return true;
            }

            let z = self[u];
            self[u] = self[v];
            u = z;
        }

        false
    }
}

impl Index<u32> for RemUnionFind {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        &self.parents[index as usize]
    }
}

impl IndexMut<u32> for RemUnionFind {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.parents[index as usize]
    }
}
