#![feature(test)]
#![allow(unused, dead_code)]
use std::time::Instant;

use anyhow::{anyhow, Result};
use clap::Parser;
use graph::FastZeroDimEdgeGenerator;
use kruskal::{rem_union_find::RemUnionFind, sized_rem_union_find::SizedRemUnionFind};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub mod bitvector;
pub mod fat_component;
pub mod graph;
pub mod kruskal;
pub mod zero_dim;

#[derive(Parser, Debug)]
#[command(name = "randmst")]
#[command(version = "1.0")]
#[command(about = "Blazingly fast implementation of CS124 progset1", long_about = None)]
struct Args {
    _ne: u32,
    #[arg(help = "Number of points per graph.")]
    num_points: u32,

    #[arg(help = "Number of trials to run.")]
    num_trials: u32,

    #[arg(value_parser= clap::value_parser!(u32).range(0..=4),
        help = "Here a `0` dimensional should be interpreted as a random complete graph.")]
    dimension: u32,

    #[arg(short, long, help = "Display total time and time per trial")]
    time: bool,

    #[arg(short, long, help = "Run each trial in series (for debugging)")]
    no_parallel: bool,
}

fn run_trial_zero_dim(num_points: u32) -> f64 {
    zero_dim::mst(num_points)
}

fn run_trial_zero_dim_2(num_points: u32) -> f64 {
    // kruskal::mst_total_length::<RemUnionFind>(FastZeroDimEdgeGenerator::new(num_points))
    kruskal::mst_total_length_fat_component::<SizedRemUnionFind>(num_points)
}

fn run_trial(num_points: u32, dimension: u32) -> f64 {
    match dimension {
        0 => run_trial_zero_dim(num_points),
        2 => run_trial_zero_dim_2(num_points),
        _ => 0.0,
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.dimension == 1 {
        return Err(anyhow!("dimension 1 is not supported!"));
    }

    let start = Instant::now();
    let average = if args.no_parallel {
        (0..args.num_trials)
            .into_iter()
            .map(|_| run_trial(args.num_points, args.dimension))
            .sum::<f64>()
    } else {
        (0..args.num_trials)
            .into_par_iter()
            .map(|_| run_trial(args.num_points, args.dimension))
            .sum::<f64>()
    } / args.num_trials as f64;
    println!("elapsed {:?}", start.elapsed());

    println!(
        "{:.6} {} {} {}",
        average, args.num_points, args.num_trials, args.dimension
    );

    Ok(())
}
