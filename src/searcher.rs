use crate::{
    grid::{Grid, SIZE},
    guess::Guess,
    string::hconcat,
};

pub struct Searcher {
    next: Grid,
    guess_stack: Vec<Guess>,
    action_stack: Vec<Action>,
    all_cells: Vec<(i32, i32)>,
    alive_cells: Vec<(i32, i32)>,
}

#[derive(Clone, Copy)]
enum Action {
    MakeGuess,
    /// If the bool is true, the first guess is that the cell is alive.
    FirstGuess(i32, i32, bool),
    /// If the bool is true, the first guess is that the cell is alive.
    SecondGuess(i32, i32, bool),
}

pub enum SearchResult {
    Found(Grid, usize),
    Working(usize),
    Unsatisfiable,
}

impl Searcher {
    pub fn new(next: Grid) -> Self {
        let alive_cells = next.alive_cells().collect::<Vec<_>>();
        let mut all_cells = (0..SIZE)
            .flat_map(|x| (0..SIZE).map(move |y| (x, y)))
            .collect::<Vec<_>>();
        // Try cells furthest from active ones first.
        all_cells.sort_by_key(|(x, y)| {
            alive_cells
                .iter()
                .map(|(ax, ay)| ((ax - x + SIZE) % SIZE).abs() + ((ay - y + SIZE) % SIZE).abs())
                .min()
                .unwrap_or(-1000)
        });

        Self {
            next,
            guess_stack: vec![Guess::default()],
            action_stack: vec![Action::MakeGuess],
            all_cells,
            alive_cells,
        }
    }

    pub fn current_guess(&self) -> Guess {
        self.guess_stack.last().unwrap().clone()
    }

    pub fn search(&mut self, max_iterations: usize) -> SearchResult {
        let mut iterations = 0;
        while let Some(action) = self.action_stack.last().copied() {
            let guess = self.guess_stack.last().unwrap();

            iterations += 1;

            match action {
                Action::MakeGuess => {
                    self.action_stack.pop();
                    // Make a guess.
                    // Pick a cell that has not yet been guessed.

                    let mut try_dead = guess.try_dead();
                    try_dead -= &guess.alive();
                    try_dead -= &guess.dead();

                    let mut try_alive = guess.try_alive();
                    try_alive -= &guess.alive();
                    try_alive -= &guess.dead();

                    match try_dead
                        .alive_cells()
                        .chain(try_alive.alive_cells())
                        .chain(self.all_cells.iter().copied().filter(|(x, y)| unsafe {
                            !guess.guessed_alive(*x, *y) && !guess.guessed_dead(*x, *y)
                        }))
                        .next()
                    {
                        Some((x, y)) => {
                            self.action_stack.push(Action::FirstGuess(
                                x,
                                y,
                                self.alive_cells.contains(&(x, y)),
                            ));
                        }
                        None => {
                            // There were no cells left to guess.
                            let mut next_grid = guess.alive();
                            next_grid.step();
                            if next_grid != self.next {
                                panic!(
                                    "grids did not match:\n{}",
                                    hconcat(
                                        &hconcat(&guess.render(), &self.next.render(), "   "),
                                        &next_grid.render(),
                                        "   "
                                    )
                                );
                            }
                            // We have a valid solution.
                            // We'll pretend to the rest of the execution procedure that this solution was invalid,
                            // so that it can keep searching.
                            let alive = guess.alive();
                            self.guess_stack.pop();
                            while let Some(action) = self.action_stack.pop() {
                                match action {
                                    Action::MakeGuess => todo!(),
                                    Action::FirstGuess(x, y, alive) => {
                                        // The last time we made a first guess,
                                        // instead do the second guess.
                                        self.action_stack.push(Action::SecondGuess(x, y, alive));
                                        break;
                                    }
                                    Action::SecondGuess(_, _, _) => {
                                        // Pop out of this inner loop too.
                                        self.guess_stack.pop();
                                    }
                                }
                            }
                            return SearchResult::Found(alive, iterations);
                        }
                    };
                }
                Action::FirstGuess(x, y, alive) => {
                    let mut new_guess = guess.clone();
                    if alive {
                        new_guess.guess_alive(&self.next, x, y);
                    } else {
                        new_guess.guess_dead(&self.next, x, y);
                    }
                    if new_guess.found_contradiction() {
                        // Instead, guess this cell was dead.
                        self.action_stack.pop();
                        self.action_stack.push(Action::SecondGuess(x, y, alive));
                    } else {
                        self.guess_stack.push(new_guess);
                        self.action_stack.push(Action::MakeGuess);
                    }
                }
                Action::SecondGuess(x, y, alive) => {
                    let mut new_guess = guess.clone();
                    if alive {
                        new_guess.guess_dead(&self.next, x, y);
                    } else {
                        new_guess.guess_alive(&self.next, x, y);
                    }
                    if new_guess.found_contradiction() {
                        // This cell can neither be dead nor alive.
                        // So `guess` is inconsistent.
                        // Pop out of this implicit loop.
                        while let Some(action) = self.action_stack.pop() {
                            match action {
                                Action::MakeGuess => unimplemented!(),
                                Action::FirstGuess(x, y, alive) => {
                                    // The last time we made a first guess,
                                    // instead do the second guess.
                                    self.action_stack.push(Action::SecondGuess(x, y, alive));
                                    break;
                                }
                                Action::SecondGuess(_, _, _) => {
                                    // Pop out of this inner loop too.
                                    self.guess_stack.pop();
                                }
                            }
                        }
                    } else {
                        self.guess_stack.push(new_guess);
                        self.action_stack.push(Action::MakeGuess);
                    }
                }
            }

            if iterations >= max_iterations {
                return SearchResult::Working(iterations);
            }
        }

        SearchResult::Unsatisfiable
    }
}
