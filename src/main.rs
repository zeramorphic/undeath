#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

use std::io::Write;

use grid::Grid;
use searcher::{SearchResult, Searcher};
use string::hconcat;

pub mod grid;
pub mod guess;
pub mod searcher;
pub mod string;

fn main() {
    let next = Grid::from_file("castle.gol");
    let mut searcher = Searcher::new(next.clone());
    let mut grids = Vec::new();
    let mut best: usize = std::usize::MAX;
    loop {
        match searcher.search(100_000) {
            SearchResult::Found(grid) => {
                if !grids.contains(&grid) {
                    let actives = grid.alive_cells().count();
                    if actives <= best {
                        println!();
                        println!(
                            "found solution #{} using {} cells:",
                            grids.len() + 1,
                            actives
                        );
                        println!("{}", hconcat(&grid.render(), &next.render(), "   "));
                        best = actives;
                    }
                    grids.push(grid);
                }
            }
            SearchResult::Working(iterations) => {
                if iterations > 1_000_000 {
                    println!("{} million iterations.", iterations / 1_000_000);
                }
                println!(
                    "{}",
                    hconcat(&searcher.current_guess().render(), &next.render(), "   ")
                );
            }
            SearchResult::Unsatisfiable => {
                println!("Search complete.");
                break;
            }
        }
    }
}
