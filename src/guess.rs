use crate::grid::{Cell, Grid, SIZE};

/// A guess for what the previous frame could look like.
#[derive(Clone)]
pub struct Guess {
    /// The grid of cells we have guessed were alive on the previous frame.
    alive: Grid,
    /// The grid of cells we have guessed were dead on the previous frame.
    dead: Grid,
    /// The minimum amount of neighbours a given cell has on the previous frame, given this guess to be correct.
    min_neighbours: Grid,
    /// The maximum amount of neighbours a given cell has on the previous frame, given this guess to be correct.
    max_neighbours: Grid,
    /// True if we know this guess leads to a logical contradiction.
    found_contradiction: bool,
    /// The cells that we want to try making alive.
    try_alive: Grid,
    /// The cells that we want to try making alive.
    try_dead: Grid,
}

impl Default for Guess {
    fn default() -> Self {
        Self {
            alive: Default::default(),
            dead: Default::default(),
            min_neighbours: Default::default(),
            max_neighbours: Grid::fill(Cell { value: 8 }),
            found_contradiction: false,
            try_alive: Default::default(),
            try_dead: Default::default(),
        }
    }
}

impl Guess {
    pub fn render(&self) -> String {
        let border = std::iter::repeat_n('─', 2 * SIZE as usize).collect::<String>();

        let column_numbers = (0..SIZE).map(|i| format!("{i:2}")).collect::<String>();
        let mut output = format!("    {column_numbers} \n   ┌{border}┐\n");
        for y in 0..SIZE {
            let mut row = String::new();
            for x in 0..SIZE {
                row += match (
                    unsafe { self.alive.get(x, y) }.value,
                    unsafe { self.dead.get(x, y) }.value,
                ) {
                    (1, _) => "██",
                    (_, 1) => "  ",
                    (_, _) => "▒▒",
                };
            }
            output += &format!("{y:2} │");
            output += &row;
            output.push('│');
            output.push('\n');
        }
        output += &format!("   └{border}┘");
        output
    }

    pub fn alive(&self) -> Grid {
        self.alive.clone()
    }

    pub fn dead(&self) -> Grid {
        self.dead.clone()
    }

    pub fn try_alive(&self) -> Grid {
        self.try_alive.clone()
    }

    pub fn try_dead(&self) -> Grid {
        self.try_dead.clone()
    }

    pub unsafe fn guessed_alive(&self, x: i32, y: i32) -> bool {
        self.alive.get(x, y).value > 0
    }

    pub unsafe fn guessed_dead(&self, x: i32, y: i32) -> bool {
        self.dead.get(x, y).value > 0
    }

    /// Adjust this guess with the additional information that the given cell is alive.
    /// Safety: `x` and `y` must be between `0` and `SIZE`.
    pub fn guess_alive(&mut self, next: &Grid, x: i32, y: i32) {
        let mut queue = Vec::with_capacity((SIZE * SIZE) as usize);
        if let Ok(()) = self.guess_alive_with_queue(x, y, &mut queue) {
            let _ = self.propagate_constraints(next, queue);
        }
    }

    /// Adjust this guess with the additional information that the given cell is dead.
    /// Safety: `x` and `y` must be between `0` and `SIZE`.
    pub fn guess_dead(&mut self, next: &Grid, x: i32, y: i32) {
        let mut queue = Vec::with_capacity((SIZE * SIZE) as usize);
        if let Ok(()) = self.guess_dead_with_queue(x, y, &mut queue) {
            let _ = self.propagate_constraints(next, queue);
        }
    }

    fn fail(&mut self) -> Result<(), ()> {
        self.found_contradiction = true;
        Err(())
    }

    fn guess_alive_with_queue(
        &mut self,
        x: i32,
        y: i32,
        queue: &mut Vec<(i32, i32)>,
    ) -> Result<(), ()> {
        if unsafe { self.dead.get(x, y) }.value > 0 {
            self.fail()?
        }

        if unsafe { self.alive.get(x, y) }.value > 0 {
            return Ok(());
        }

        unsafe {
            self.alive.set(x, y, Cell::one());
        }

        // Now preserve the invariant that `min_neighbours` and `max_neighbours` are correct.
        // For each proper neighbour of the cell, increase `min_neighbours` by one.
        for (x2, y2) in Grid::neighbour_positions(x, y) {
            unsafe {
                self.min_neighbours.set_add(x2, y2, Cell::one());
            }
            // It's faster without this check!
            // if !queue.contains(&(x2, y2)) {
            queue.push((x2, y2));
            // }
        }

        Ok(())
    }

    fn guess_dead_with_queue(
        &mut self,
        x: i32,
        y: i32,
        queue: &mut Vec<(i32, i32)>,
    ) -> Result<(), ()> {
        if unsafe { self.alive.get(x, y) }.value > 0 {
            self.fail()?
        }

        if unsafe { self.dead.get(x, y) }.value > 0 {
            return Ok(());
        }

        unsafe {
            self.dead.set(x, y, Cell::one());
        }

        // Now preserve the invariant that `min_neighbours` and `max_neighbours` are correct.
        // For each proper neighbour of the cell, decrease `max_neighbours` by one.
        for (x2, y2) in Grid::neighbour_positions(x, y) {
            unsafe {
                self.max_neighbours.set_add(x2, y2, Cell::neg_one());
            }
            // if !queue.contains(&(x2, y2)) {
            queue.push((x2, y2));
            // }
        }

        Ok(())
    }

    fn guess_neighbours_alive_with_queue(
        &mut self,
        x: i32,
        y: i32,
        queue: &mut Vec<(i32, i32)>,
    ) -> Result<(), ()> {
        for (x2, y2) in Grid::neighbour_positions(x, y) {
            // If the cell is not already marked as dead or alive...
            if unsafe { self.dead.get(x2, y2) }.value == 0
                && unsafe { self.alive.get(x2, y2) }.value == 0
            {
                // ...guess that it is alive.
                self.guess_alive_with_queue(x2, y2, queue)?;
            }
        }
        Ok(())
    }

    fn guess_neighbours_dead_with_queue(
        &mut self,
        x: i32,
        y: i32,
        queue: &mut Vec<(i32, i32)>,
    ) -> Result<(), ()> {
        for (x2, y2) in Grid::neighbour_positions(x, y) {
            // If the cell is not already marked as dead or alive...
            if unsafe { self.dead.get(x2, y2) }.value == 0
                && unsafe { self.alive.get(x2, y2) }.value == 0
            {
                // ...guess that it is dead.
                self.guess_dead_with_queue(x2, y2, queue)?;
            }
        }
        Ok(())
    }

    /// Given information about minimum and maximum neighbours, and the next frame of the grid,
    /// work out some more information about the previous frame.
    /// The queue is the list of cells whose neighbour count has just been updated.
    /// Safety: this queue should only contain properly wrapped positions.
    fn propagate_constraints(&mut self, next: &Grid, mut queue: Vec<(i32, i32)>) -> Result<(), ()> {
        while let Some((x, y)) = queue.pop() {
            let min = unsafe { self.min_neighbours.get(x, y) }.value;
            let max = unsafe { self.max_neighbours.get(x, y) }.value;
            let next_state = unsafe { next.get(x, y) }.value;

            if min == max {
                // Naive approach.
                match min {
                    3 => {
                        // The cell must be alive next frame.
                        if next_state == 0 {
                            return self.fail();
                        }
                    }
                    2 => {
                        if next_state > 0 {
                            // The cell is alive next frame.
                            // So it must be alive on the previous frame.
                            self.guess_alive_with_queue(x, y, &mut queue)?;
                        } else {
                            // The cell is dead next frame.
                            // So it must be dead on the previous frame.
                            self.guess_dead_with_queue(x, y, &mut queue)?;
                        }
                    }
                    _ => {
                        // The cell must be dead next frame.
                        if next_state > 0 {
                            return self.fail();
                        }
                    }
                }
                continue;
            }

            // Min/max calculations.
            if max < 2 || min > 3 {
                // This cell *must* be dead on the next frame.
                // The value of this cell in the previous frame is arbitrary.
                if next_state > 0 {
                    return self.fail();
                }
            } else if max == 2 {
                if next_state > 0 {
                    // This cell can only be alive on the next frame if it is alive on the previous frame.
                    self.guess_alive_with_queue(x, y, &mut queue)?;
                    // Also, in this case, all of its remaining neighbours are forced to be alive on the previous frame.
                    // This way we enforce the maximum.
                    self.guess_neighbours_alive_with_queue(x, y, &mut queue)?;
                } else if min == 2 {
                    // The cell is dead on the next frame and has exactly 2 neighbours.
                    // The only way this can happen is if it is dead on the previous frame.
                    self.guess_dead_with_queue(x, y, &mut queue)?;
                } else {
                    // This cell is dead on the next frame and has 0, 1, or 2 neighbours.
                    // We can't make a deduction now.
                }
            } else if min == 3 {
                if next_state > 0 {
                    // This cell can only be alive on the next frame if it has exactly three living neighbours.
                    // Force the remaining undecided neighbours to be dead.
                    // We can't tell if this cell is alive or dead on the previous frame.
                    self.guess_neighbours_dead_with_queue(x, y, &mut queue)?;
                } else if max == 4 {
                    // This cell is dead on the next frame and has at least three neighbours.
                    // The only way this can happen is if it in fact has four neighbours.
                    // Given `max == 4`, there must be exactly one undecided neighbour left.
                    for (x2, y2) in Grid::neighbour_positions(x, y) {
                        // If the cell is not already marked as dead or alive...
                        if unsafe { self.dead.get(x2, y2) }.value == 0
                            && unsafe { self.alive.get(x2, y2) }.value == 0
                        {
                            // ...guess that it is alive.
                            self.guess_alive_with_queue(x2, y2, &mut queue)?;
                            // We already know that there is exactly one neighbour.
                            break;
                        }
                    }
                }
            }

            // Previous state calculations.
            // Helpful for adding constraints to cells that live on the next frame.
            let previously_dead = unsafe { self.dead.get(x, y) }.value;
            let previously_alive = unsafe { self.alive.get(x, y) }.value;

            match (next_state > 0, previously_dead > 0, previously_alive > 0) {
                (true, true, _) => {
                    // This cell comes alive on the next frame.
                    // It can only do this if it has exactly three neighbours.
                    if min == 3 {
                        self.guess_neighbours_dead_with_queue(x, y, &mut queue)?;
                    } else if max == 3 {
                        self.guess_neighbours_alive_with_queue(x, y, &mut queue)?;
                    } else if min == 2 {
                        // We'd like to try individually setting the neighbours of this cell to be alive.
                        for (x2, y2) in Grid::neighbour_positions(x, y) {
                            // If the cell is not already marked as dead or alive...
                            if unsafe { self.dead.get(x2, y2) }.value == 0
                                && unsafe { self.alive.get(x2, y2) }.value == 0
                            {
                                // ...express our desire to test the case where it is alive.
                                unsafe {
                                    self.try_alive.set(x2, y2, Cell::one());
                                }
                            }
                        }
                    } else if max == 4 {
                        // We'd like to try individually setting the neighbours of this cell to be dead.
                        for (x2, y2) in Grid::neighbour_positions(x, y) {
                            if unsafe { self.dead.get(x2, y2) }.value == 0
                                && unsafe { self.alive.get(x2, y2) }.value == 0
                            {
                                unsafe {
                                    self.try_dead.set(x2, y2, Cell::one());
                                }
                            }
                        }
                    }
                }
                (true, _, true) => {
                    // This cell remains alive.
                    // It can only do this if it has exactly two or three neighbours.
                    if min == 3 {
                        self.guess_neighbours_dead_with_queue(x, y, &mut queue)?;
                    } else if max == 2 {
                        self.guess_neighbours_alive_with_queue(x, y, &mut queue)?;
                    }
                }
                (false, true, _) => {
                    // This cell remains dead.
                    // There are lots of ways this can happen.
                    // TODO
                }
                (false, _, true) => {
                    // This cell dies.
                    // It only does this if it does *not* have exactly two or three neighbours.
                    // TODO
                }
                (_, _, _) => {}
            }
        }
        Ok(())
    }

    pub fn found_contradiction(&self) -> bool {
        self.found_contradiction
    }
}
