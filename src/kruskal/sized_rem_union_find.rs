use std::ops::{Index, IndexMut};

use super::SizedKruskalUnionFind;

pub struct SizedRemUnionFind {
    parents: Vec<u32>,
    sizes: Vec<u32>,
}

impl SizedKruskalUnionFind for SizedRemUnionFind {
    fn new(size: u32) -> Self {
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
                v = self.find(v);
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

    fn same_set(&mut self, mut u: u32, mut v: u32) -> bool {
        let u_orig = u;
        let v_orig = v;

        if (u == self[u]) && (v == self[v]) {
            return u == v;
        }

        while (u != self[u]) || (v != self[v]) {
            u = self[u];
            v = self[v];
        }

        // Path splitting step
        self[u_orig] = u;
        self[v_orig] = v;

        u == v
    }

    fn find(&mut self, mut u: u32) -> u32 {
        // Path splitting
        let u_orig = u;

        while u != self[u] {
            u = self[u];
        }

        // Path splitting step
        self[u_orig] = u;

        u
    }

    fn size(&mut self, u: u32) -> u32 {
        let root = self.find(u);
        self.sizes[root as usize]
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
