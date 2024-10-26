use std::{
    mem::MaybeUninit,
    ops::{Add, AddAssign, Sub, SubAssign},
    path::Path,
};

pub const SIZE: i32 = 12;

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Cell {
    pub value: i32,
}

impl Cell {
    #[inline]
    pub fn neg_one() -> Cell {
        Cell { value: -1 }
    }

    #[inline]
    pub fn zero() -> Cell {
        Cell { value: 0 }
    }

    #[inline]
    pub fn one() -> Cell {
        Cell { value: 1 }
    }
}

impl Add for Cell {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl Sub for Cell {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value - rhs.value,
        }
    }
}

impl AddAssign for Cell {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl SubAssign for Cell {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

/// A toroidal grid for Life.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Grid {
    /// Row-major.
    /// A cell (x, y) is at `x + y * SIZE`.
    cells: [Cell; (SIZE * SIZE) as usize],
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cells: unsafe { std::mem::zeroed() },
        }
    }
}

impl<'a> AddAssign<&'a Grid> for Grid {
    #[inline]
    fn add_assign(&mut self, rhs: &'a Self) {
        for i in 0..(SIZE * SIZE) as usize {
            unsafe {
                *self.cells.get_unchecked_mut(i) += *rhs.cells.get_unchecked(i);
            }
        }
    }
}

impl<'a> SubAssign<&'a Grid> for Grid {
    #[inline]
    fn sub_assign(&mut self, rhs: &'a Grid) {
        for i in 0..(SIZE * SIZE) as usize {
            unsafe {
                *self.cells.get_unchecked_mut(i) -= *rhs.cells.get_unchecked(i);
            }
        }
    }
}

impl Grid {
    pub fn fill(cell: Cell) -> Self {
        Self {
            cells: [cell; (SIZE * SIZE) as usize],
        }
    }

    #[inline]
    pub unsafe fn get(&self, x: i32, y: i32) -> Cell {
        *self.cells.get_unchecked((x + y * SIZE) as usize)
    }

    #[inline]
    pub unsafe fn set(&mut self, x: i32, y: i32, cell: Cell) {
        *self.cells.get_unchecked_mut((x + y * SIZE) as usize) = cell;
    }

    /// Assumes `x` and `y` aren't less than or equal to `-SIZE`.
    #[inline]
    pub fn get_wrapped(&self, x: i32, y: i32) -> Cell {
        unsafe { self.get((x + SIZE) % SIZE, (y + SIZE) % SIZE) }
    }

    /// Assumes `x` and `y` aren't less than or equal to `-SIZE`.
    #[inline]
    pub fn set_wrapped(&mut self, x: i32, y: i32, cell: Cell) {
        unsafe { self.set((x + SIZE) % SIZE, (y + SIZE) % SIZE, cell) }
    }

    #[inline]
    pub unsafe fn set_add(&mut self, x: i32, y: i32, cell: Cell) {
        *self.cells.get_unchecked_mut((x + y * SIZE) as usize) += cell;
    }

    /// Assumes `x` and `y` aren't less than or equal to `-SIZE`.
    #[inline]
    pub fn set_add_wrapped(&mut self, x: i32, y: i32, cell: Cell) {
        let x = (x + SIZE) % SIZE;
        let y = (y + SIZE) % SIZE;
        unsafe {
            self.set_add(x, y, cell);
        }
    }

    /// The list of proper neighbours.
    /// All clamped to `0..SIZE`.
    #[inline]
    pub fn neighbour_positions(x: i32, y: i32) -> [(i32, i32); 8] {
        [
            ((x + -1 + SIZE) % SIZE, (y + -1 + SIZE) % SIZE),
            ((x + -1 + SIZE) % SIZE, (y + 0 + SIZE) % SIZE),
            ((x + -1 + SIZE) % SIZE, (y + 1 + SIZE) % SIZE),
            ((x + 0 + SIZE) % SIZE, (y + -1 + SIZE) % SIZE),
            ((x + 0 + SIZE) % SIZE, (y + 1 + SIZE) % SIZE),
            ((x + 1 + SIZE) % SIZE, (y + -1 + SIZE) % SIZE),
            ((x + 1 + SIZE) % SIZE, (y + 0 + SIZE) % SIZE),
            ((x + 1 + SIZE) % SIZE, (y + 1 + SIZE) % SIZE),
        ]
    }

    pub fn alive_cells(&self) -> impl Iterator<Item = (i32, i32)> + use<'_> {
        (0..SIZE)
            .flat_map(|x| (0..SIZE).map(move |y| (x, y)))
            .filter(|(x, y)| unsafe { self.get(*x, *y) }.value > 0)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        let mut result = Self::default();
        for (y, line) in contents.lines().enumerate() {
            for (x, char) in line.chars().enumerate() {
                let cell = Cell {
                    value: if " .".contains(char) { 0 } else { 1 },
                };
                result.set_wrapped(x as i32, y as i32, cell);
            }
        }
        result
    }

    /// Renders the grid to a string.
    pub fn render(&self) -> String {
        let border = std::iter::repeat_n('─', 2 * SIZE as usize).collect::<String>();

        let column_numbers = (0..SIZE).map(|i| format!("{i:2}")).collect::<String>();
        let mut output = format!("    {column_numbers} \n   ┌{border}┐\n");
        for y in 0..SIZE {
            let mut row = String::new();
            for x in 0..SIZE {
                row += match unsafe { self.get(x, y) }.value {
                    0 => "  ",
                    _ => "██",
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

    /// Work out the amount of neighbours of this cell.
    pub fn neighbours(&self) -> Self {
        let mut neighbours = self.hcount().vcount();
        neighbours -= self;
        neighbours
    }

    /// Compute the next step of the simulation.
    pub fn step(&mut self) {
        let neighbours = self.neighbours();
        for x in 0..SIZE {
            for y in 0..SIZE {
                let current = unsafe { self.get(x, y) };
                let count_neighbours = unsafe { neighbours.get(x, y) };
                // The rules are that the output cell is alive if:
                // 1. count_neighbours = 3, or
                // 2. count_neighbours = 2 and current is > 0.
                let new_value = Cell {
                    value: match count_neighbours.value {
                        3 => 1,
                        2 => {
                            if current.value > 0 {
                                1
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    },
                };
                unsafe {
                    self.set(x, y, new_value);
                }
            }
        }
    }

    /// Each output cell is the sum of the values of the input cell and its vertical neighbours.
    fn vcount(&self) -> Self {
        let mut result: [MaybeUninit<Cell>; (SIZE * SIZE) as usize] = MaybeUninit::uninit_array();

        for x in 0..SIZE {
            for y in 0..SIZE {
                result[(x + y * SIZE) as usize].write(
                    self.get_wrapped(x, y - 1)
                        + self.get_wrapped(x, y)
                        + self.get_wrapped(x, y + 1),
                );
            }
        }

        Grid {
            cells: unsafe { MaybeUninit::array_assume_init(result) },
        }
    }

    /// Each output cell is the sum of the values of the input cell and its horizontal neighbours.
    fn hcount(&self) -> Self {
        let mut result: [MaybeUninit<Cell>; (SIZE * SIZE) as usize] = MaybeUninit::uninit_array();

        for x in 0..SIZE {
            for y in 0..SIZE {
                result[(x + y * SIZE) as usize].write(
                    self.get_wrapped(x - 1, y)
                        + self.get_wrapped(x, y)
                        + self.get_wrapped(x + 1, y),
                );
            }
        }

        Grid {
            cells: unsafe { MaybeUninit::array_assume_init(result) },
        }
    }
}
