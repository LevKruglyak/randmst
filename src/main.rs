use anyhow::{anyhow, Result};
use clap::Parser;
use graph::FastZeroDimEdgeGenerator;
use kruskal::rem_union_find::RemUnionFind;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub mod graph;
pub mod kruskal;

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
    #[arg(
        short,
        long,
        default_value_t = true,
        help = "Run each trial in parrallel (default true)"
    )]
    parallel: bool,
}

fn run_trial_zero_dim(num_points: u32) -> f64 {
    kruskal::mst_total_length::<RemUnionFind>(FastZeroDimEdgeGenerator::new(num_points))
}

fn run_trial(num_points: u32, dimension: u32) -> f64 {
    let start = Instant::now();
    match dimension {
        0 => run_trial_zero_dim(num_points),
        _ => 0.0,
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.dimension == 1 {
        return Err(anyhow!("dimension 1 is not supported!"));
    }

    let average = if args.parallel {
        (0..args.num_trials)
            .into_par_iter()
            .map(|_| run_trial(args.num_points, args.dimension))
            .sum::<f64>()
    } else {
        (0..args.num_trials)
            .into_iter()
            .map(|_| run_trial(args.num_points, args.dimension))
            .sum::<f64>()
    } / args.num_trials as f64;

    println!(
        "{:.4} {} {} {}",
        average, args.num_points, args.num_trials, args.dimension
    );

    Ok(())
}
