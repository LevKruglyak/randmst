#![feature(test)]
// #![allow(unused, dead_code)]
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use average::MeanWithError;
use clap::Parser;
use colored::Colorize;
use rand::{thread_rng, RngCore};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

pub mod complete;
pub mod euclidean;

#[derive(Parser, Debug)]
#[command(name = "randmst")]
#[command(version = "1.0")]
#[command(about = "Blazingly fast sampler of minimum spanning trees of a random (Euclidean) complete graph.", long_about = None)]
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

    #[arg(short, long, help = "Also display the error of the result")]
    error: bool,
}

fn run_trial_zero_dim(num_points: u32, rng: impl RngCore) -> f64 {
    complete::mst(num_points, rng)
}

fn run_trial(num_points: u32, dimension: u32, rng: impl RngCore) -> (f64, Duration) {
    let start = Instant::now();
    let mst = match dimension {
        0 => run_trial_zero_dim(num_points, rng),
        _ => 0.0,
    };
    (mst, start.elapsed())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.dimension == 1 {
        return Err(anyhow!("dimension 1 is not supported!"));
    }

    // Run the trials
    let timed_trials: Vec<(f64, Duration)> = if args.no_parallel {
        (0..args.num_trials)
            .into_iter()
            .map(|_| run_trial(args.num_points, args.dimension, thread_rng()))
            .collect()
    } else {
        (0..args.num_trials)
            .into_par_iter()
            .map(|_| run_trial(args.num_points, args.dimension, thread_rng()))
            .collect()
    };

    // Calculate average and variance
    let average: MeanWithError = timed_trials.iter().map(|x| x.0).collect();

    // Display time calculations
    if args.time {
        let average_time: MeanWithError = timed_trials.iter().map(|x| x.1.as_secs_f64()).collect();
        let mean = Duration::from_secs_f64(average_time.mean());
        let error = Duration::from_secs_f64(average_time.error());
        println!(
            "time per trial: {} ± {}",
            format!("{mean:?}").green(),
            format!("{error:?}").red(),
        );
    }

    // Decide how to format result
    let result = if args.error {
        format!(
            "{} ± {}",
            format!("{:.6}", average.mean()).green(),
            format!("{:.6}", average.error()).red()
        )
    } else {
        format!("{:.6}", average.mean())
    };

    println!(
        "{} {} {} {}",
        result, args.num_points, args.num_trials, args.dimension
    );

    Ok(())
}
