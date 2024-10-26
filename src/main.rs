#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

use std::time::Duration;

use grid::Grid;

pub mod grid;

fn main() {
    let mut grid = Grid::from_file("glider.gol");

    loop {
        println!("{}", grid.render());
        std::thread::sleep(Duration::from_millis(200));
        grid.step();
    }
}
