#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

use grid::Grid;
use searcher::{SearchResult, Searcher};
use string::large_number;

pub mod grid;
pub mod guess;
pub mod searcher;
pub mod string;

#[derive(Clone)]
pub struct Sequence {
    grids: Vec<Grid>,
    terminated: bool,
}

fn main() {
    let start = Grid::from_file("glider.gol");
    let mut attempts = vec![Sequence {
        grids: vec![start],
        terminated: false,
    }];

    let micro_step_size = 100_000;
    let mut macro_steps = 0;
    let mut i = 0;
    loop {
        let current_attempt = &mut attempts[i];
        if !current_attempt.terminated {
            let mut searcher = Searcher::new(current_attempt.grids.last().unwrap().clone());
            match searcher.search(micro_step_size) {
                SearchResult::Found(grid, iterations) => {
                    let mut new_attempt = current_attempt.clone();
                    new_attempt.grids.push(grid);
                    attempts.push(new_attempt);
                }
                SearchResult::Working(iterations) => {
                    // println!("{} million iterations.", iterations / 1_000_000);
                    // println!(
                    //     "{}",
                    //     hconcat(&searcher.current_guess().render(), &next.render(), "   ")
                    // );
                }
                SearchResult::Unsatisfiable => {
                    current_attempt.terminated = true;
                }
            }
        }

        i += 1;
        if i >= attempts.len() {
            i = 0;
        }

        macro_steps += 1;

        if macro_steps % 10 == 0 {
            println!("---");
            println!(
                "{} iterations.",
                large_number(macro_steps * micro_step_size)
            );
            println!("{} running attempts.", large_number(attempts.len()));
            println!(
                "{} terminated attempts.",
                attempts.iter().filter(|x| x.terminated).count()
            );
            println!(
                "Longest chain is length {}.",
                attempts.iter().map(|x| x.grids.len()).max().unwrap()
            );
        }
    }
}
