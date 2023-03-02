use rand::{Rng, RngCore};
use rand_distr::Exp1;

use crate::Edge;

use super::union_find::{Point, SizedUnionFind};

pub struct FatComponent {
    pub root: Point,
    pub size: u32,
    pub remainders: Vec<Point>,
}

impl FatComponent {
    fn new(root: Point, size: u32) -> Self {
        Self {
            root,
            size,
            remainders: Vec::new(),
        }
    }
}

pub struct FatComponentSampler<R: RngCore> {
    inv_weight: f64,
    fat_component: Option<FatComponent>,
    total_count: u32,
    set: SizedUnionFind,
    rng: R,
}

impl<R: RngCore> Iterator for FatComponentSampler<R> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        if self.set.free_edges() == 0 || self.total_count == 0 {
            return None;
        }

        // When `free_edges` large enough, we can use an approximate distribution
        // to speed up computation
        if self.set.free_edges() > (1 << 16) {
            self.inv_weight -= self.rng.sample::<f64, _>(Exp1) / self.set.free_edges() as f64;
        } else {
            self.inv_weight *=
                (-self.rng.sample::<f64, _>(Exp1) / self.set.free_edges() as f64).exp();
        }

        // Update the fat component
        if let Some(component) = self.fat_component.as_mut() {
            update_fat_component(&mut self.set, component)
        }

        // If we haven't found a fat component yet, look for it!
        if (self.set.free_edges() * 2 < self.set.total_edges()) & self.fat_component.is_none() {
            self.fat_component = find_fat_component(&mut self.set);
        }

        loop {
            let edge = match &self.fat_component {
                Some(component) => sample_component_edge(&mut self.rng, &mut self.set, component),
                None => sample_sparse_edge(&mut self.rng, &mut self.set),
            };

            if !self.set.unite(edge.0, edge.1) {
                continue;
            }

            self.total_count -= 1;

            return Some(Edge {
                u: edge.0.as_u32(),
                v: edge.1.as_u32(),
                w: 1.0 - self.inv_weight,
            });
        }
    }
}

impl<R: RngCore> FatComponentSampler<R> {
    pub fn new(rng: R, size: u32) -> Self {
        Self {
            inv_weight: 1.0,
            rng,
            set: SizedUnionFind::new(size),
            total_count: size - 1,
            fat_component: None,
        }
    }
}

fn find_fat_component(set: &mut SizedUnionFind) -> Option<FatComponent> {
    for v in set.iter() {
        if set.size(v) * 2 >= set.total_size() {
            let root = set.root(v);

            let mut fat_component = FatComponent::new(root, set.size(v));
            for w in set.iter() {
                if set.root(w) != root {
                    fat_component.remainders.push(w);
                }
            }

            return Some(fat_component);
        }
    }

    None
}

fn update_fat_component(set: &mut SizedUnionFind, component: &mut FatComponent) {
    // Update location and size of the fat component
    component.root = set.root(component.root);
    component.size = set.size(component.root);

    if (set.total_size() - component.size) * 2 < component.remainders.len() as u32 {
        // Marginally faster than retain (filter rate is too low for retain to be effective)
        component.remainders = component
            .remainders
            .iter()
            .copied()
            .filter(|point| set.root(*point) != component.root)
            .collect();

        // component
        //     .remainders
        //     .retain(|point| set.root(*point) != component.root);
    }
}

fn sample_sparse_edge(rng: &mut impl RngCore, set: &mut SizedUnionFind) -> (Point, Point) {
    loop {
        // TODO: figure out trait issue
        let u = rng.sample(&*set);
        let v = rng.sample(&*set);
        if !set.same_set(u, v) {
            return (u, v);
        }
    }
}

#[allow(clippy::needless_return)]
fn sample_component_edge(
    rng: &mut impl RngCore,
    set: &mut SizedUnionFind,
    component: &FatComponent,
) -> (Point, Point) {
    let fat_size = component.size;
    let remainder_size = set.total_size() - fat_size;
    let active = set.free_edges();

    if rng.gen_bool((fat_size as usize * remainder_size as usize) as f64 / active as f64) {
        let u = sample_component(rng, set, component);
        let v = sample_remainder(rng, set, component); // assert!(!set.same_set(u, v));
                                                       // assert!(!set.same_set(u, v));
        return (u, v);
    } else {
        loop {
            let u = sample_remainder(rng, set, component);
            let v = sample_remainder(rng, set, component);
            if !set.same_set(u, v) {
                return (u, v);
            }
        }
    };
}

fn sample_component(
    rng: &mut impl RngCore,
    set: &mut SizedUnionFind,
    component: &FatComponent,
) -> Point {
    loop {
        let u = rng.sample(&*set);
        if set.root(u) == component.root {
            return u;
        }
    }
}

fn sample_remainder(
    rng: &mut impl RngCore,
    set: &mut SizedUnionFind,
    component: &FatComponent,
) -> Point {
    loop {
        let index = rng.gen_range(0..component.remainders.len());
        let u = component.remainders[index];
        if set.root(u) != component.root {
            return u;
        }
    }
}
