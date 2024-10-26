#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::SystemTime,
};

use chrono::{DateTime, Local};
use grid::Grid;
use rand::seq::SliceRandom;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use searcher::{SearchResult, Searcher};
use string::large_number;

pub mod grid;
pub mod guess;
pub mod searcher;
pub mod string;

pub struct Sequence {
    grids: Vec<Grid>,
    searcher: Searcher,
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .unwrap();

    let start = Grid::from_file("minicastle.gol");
    let mut attempts = vec![Sequence {
        grids: vec![start.clone()],
        searcher: Searcher::new(start),
    }];

    let mut macro_step = 0;
    let total_iterations: Arc<AtomicUsize> = Default::default();
    let micro_step_size = 100_000;
    let max_attempts = 100;

    let prefix = DateTime::<Local>::from(SystemTime::now())
        .format("out-%Y-%m-%d-%H-%M-%S")
        .to_string();
    std::fs::create_dir(&prefix).unwrap();

    let terminated_attempts = Arc::new(Mutex::new(Vec::new()));
    let terminated_attempts2 = Arc::clone(&terminated_attempts);

    let mut rng = rand::thread_rng();
    loop {
        attempts.shuffle(&mut rng);
        // Search for the attempts with the smallest amount of alive cells first.
        attempts.sort_by_cached_key(|x| {
            x.grids.last().unwrap().alive_cells().count() as i32 - x.grids.len() as i32
        });
        attempts = attempts
            .into_par_iter()
            .take(max_attempts)
            .flat_map(|mut current_attempt| {
                match current_attempt.searcher.search(micro_step_size) {
                    SearchResult::Found(grid, iterations) => {
                        total_iterations.fetch_add(iterations, Ordering::SeqCst);
                        let mut new_grids = current_attempt.grids.clone();
                        new_grids.push(grid.clone());
                        vec![
                            current_attempt,
                            Sequence {
                                grids: new_grids,
                                searcher: Searcher::new(grid),
                            },
                        ]
                    }
                    SearchResult::Working(iterations) => {
                        total_iterations.fetch_add(iterations, Ordering::SeqCst);
                        // println!("{} million iterations.", iterations / 1_000_000);
                        // println!(
                        //     "{}",
                        //     hconcat(&searcher.current_guess().render(), &next.render(), "   ")
                        // );
                        vec![current_attempt]
                    }
                    SearchResult::Unsatisfiable => {
                        terminated_attempts2.lock().unwrap().push(current_attempt);
                        Vec::new()
                    }
                }
            })
            .collect::<Vec<_>>();

        macro_step += 1;

        println!("---");
        println!("Macrostep #{macro_step:06}.");
        println!(
            "{} iterations.",
            large_number(total_iterations.load(Ordering::SeqCst))
        );
        println!("{} running attempts.", large_number(attempts.len()));
        println!(
            "{} terminated attempts.",
            terminated_attempts.lock().unwrap().len()
        );
        let guard = terminated_attempts.lock().unwrap();
        let best_sequence = attempts
            .iter()
            .chain(guard.iter())
            .max_by_key(|x| x.grids.len())
            .unwrap();
        println!("Longest chain is length {}.", best_sequence.grids.len());
        std::fs::write(
            format!("{prefix}/{macro_step:06}.txt"),
            best_sequence
                .grids
                .iter()
                .map(|x| x.render())
                .collect::<Vec<_>>()
                .join("\n\n\n"),
        )
        .unwrap();
        drop(guard);
    }
}
