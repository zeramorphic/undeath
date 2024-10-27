# Conway's game of 𝔲𝔫𝔡𝔢𝔞𝔱𝔥

Ever wanted to reverse engineer the meaning of life?
Well, this repository is for you!
We turn a target state of a Game of Life board into a previous one, many times over, creating a rich branching history.
And our code's homegrown and organic - none of that factory-farmed SMT stuff.

## Running the code

1. Have Rust installed.
2. Edit the thread pool size in `main.rs:30` to your liking.
3. Choose a target grid, and write it in `main.rs:34`.
4. Make sure that the grid size in `grid.rs:7` matches your intended dimensions.
5. Optionally, edit the heuristic in `searcher.rs:38`; if a grid doesn't work, try flipping that minus sign to a plus sign.
6. Run `cargo run --release`, and watch in the `out-*` directory for some dumped output!
