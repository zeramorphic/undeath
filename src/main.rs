#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

use grid::{Grid, SIZE};
use guess::Guess;
use string::hconcat;

pub mod grid;
pub mod guess;
pub mod string;

#[derive(Clone, Copy)]
enum Action {
    MakeGuess,
    /// If the bool is true, the first guess is that the cell is alive.
    FirstGuess(i32, i32, bool),
    /// If the bool is true, the first guess is that the cell is alive.
    SecondGuess(i32, i32, bool),
}

fn main() {
    let next = Grid::from_file("glider.gol");

    let mut guess_stack = vec![Guess::default()];
    let mut action_stack = vec![Action::MakeGuess];

    let alive_cells = next.alive_cells().collect::<Vec<_>>();
    let mut all_cells = (0..SIZE)
        .flat_map(|x| (0..SIZE).map(move |y| (x, y)))
        .collect::<Vec<_>>();
    // Try cells closer to active ones first.
    all_cells.sort_by_key(|(x, y)| {
        alive_cells
            .iter()
            .map(|(ax, ay)| (ax - x).abs() + (ay - y).abs())
            .min()
    });

    let mut iterations = 0;
    while let Some(action) = action_stack.last().copied() {
        let guess = guess_stack.last().unwrap();

        iterations += 1;
        if iterations % 100_000 == 0 {
            println!("ITERATION {iterations}");
            println!("{}", hconcat(&guess.render(), &next.render(), "   "));
        }

        match action {
            Action::MakeGuess => {
                action_stack.pop();
                // Make a guess.
                // Pick a cell that has not yet been guessed.
                match all_cells.iter().find(|(x, y)| unsafe {
                    !guess.guessed_alive(*x, *y) && !guess.guessed_dead(*x, *y)
                }) {
                    Some((x, y)) => {
                        action_stack.push(Action::FirstGuess(
                            *x,
                            *y,
                            alive_cells.contains(&(*x, *y)),
                        ));
                    }
                    None => {
                        // There were no cells left to guess.
                        let mut next_grid = guess.alive();
                        next_grid.step();
                        println!("SATISFIED in {iterations} iterations");
                        println!(
                            "{}",
                            hconcat(
                                &hconcat(&guess.render(), &next.render(), "   "),
                                &next_grid.render(),
                                "   "
                            )
                        );
                        break;
                    }
                }
            }
            Action::FirstGuess(x, y, alive) => {
                let mut new_guess = guess.clone();
                if alive {
                    new_guess.guess_alive(&next, x, y);
                } else {
                    new_guess.guess_dead(&next, x, y);
                }
                if new_guess.found_contradiction() {
                    // Instead, guess this cell was dead.
                    action_stack.pop();
                    action_stack.push(Action::SecondGuess(x, y, alive));
                } else {
                    guess_stack.push(new_guess);
                    action_stack.push(Action::MakeGuess);
                }
            }
            Action::SecondGuess(x, y, alive) => {
                let mut new_guess = guess.clone();
                if alive {
                    new_guess.guess_dead(&next, x, y);
                } else {
                    new_guess.guess_alive(&next, x, y);
                }
                if new_guess.found_contradiction() {
                    // This cell can neither be dead nor alive.
                    // So `guess` is inconsistent.
                    // Pop out of this implicit loop.
                    while let Some(action) = action_stack.pop() {
                        match action {
                            Action::MakeGuess => todo!(),
                            Action::FirstGuess(x, y, alive) => {
                                // The last time we made a first guess,
                                // instead do the second guess.
                                action_stack.push(Action::SecondGuess(x, y, alive));
                                break;
                            }
                            Action::SecondGuess(_, _, _) => {
                                // Pop out of this inner loop too.
                                guess_stack.pop();
                            }
                        }
                    }
                } else {
                    guess_stack.push(new_guess);
                    action_stack.push(Action::MakeGuess);
                }
            }
        }
    }


    println!("SEARCH TERMINATED after {iterations} iterations");
}
