use std::collections::HashSet;

fn main() {
    let board = default_board();
    print_board(&board);
    let mut sudoku = Sudoku::new(board);
    if let Some(solved_board) = sudoku.solve() {
        print_board(&solved_board);
    } else {
        println!("No solution found.");
    }
}

/// Represents a cell in a sudoku board. It may be solved, in which case
/// `solution` needs to be some number, and `candidates`, `candidate`, and
/// `candidate_idx` need be None; or it's unsolved in which case the above
/// relationship is reversed.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cell {
    solution: Option<i8>,
    candidates: HashSet<i8>,
    candidate: Option<i8>,
    candidate_idx: Option<usize>,
}

impl Cell {
    pub fn solved(solution: i8) -> Cell {
        Cell {
            solution: Some(solution),
            candidates: HashSet::new(),
            candidate: None,
            candidate_idx: None,
        }
    }

    pub fn unsolved() -> Cell {
        Cell {
            solution: None,
            candidates: HashSet::new(),
            candidate: None,
            candidate_idx: None,
        }
    }
}

pub type Board = [[Cell; 9]; 9];

pub struct Sudoku {
    board: Board,
    blocks: [Block; 9],
}

impl Sudoku {
    pub fn new(board: Board) -> Sudoku {
        let blocks = make_blocks(&board);
        Sudoku {
            board: board,
            blocks: blocks,
        }
    }

    /// If the board passed to the constructor is solvable, it returns a copy of
    /// the solved board. If it's unsolvable, None is returned.
    pub fn solve(&mut self) -> Option<Board> {
        self.find_candidates();
        self.guess_solutions()
    }

    /// Narrows down the search-space by assigning valid candidates to each cell
    /// and marks cells as solved that only have a single candidate.
    fn find_candidates(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                // Skip solved cells.
                if self.board[row][col].solution.is_some() {
                    continue;
                }

                let candidates = self.find_cell_candidates(row, col);
                if candidates.len() == 1 {
                    // We have a solution for this cell.
                    let solution = *candidates.iter().next().unwrap();
                    self.found_solution(row, col, solution);
                } else if !candidates.is_empty() {
                    self.board[row][col].candidates = candidates;
                }
            }
        }
    }

    /// Finds all possible candidates for a cell by checking solved cells in the
    /// same row, column, and its block.
    fn find_cell_candidates(&self, row: usize, col: usize) -> HashSet<i8> {
        let mut candidates = HashSet::new();
        let block = &self.blocks[block_index(row, col)];
        assert!(block.solutions.len() < 9);

        'candidate_selection: for candidate in 1..10 {
            // Don't add to candidates if already in block.
            if block.solutions.iter().any(|solved| *solved == candidate) {
                continue;
            }

            // Disregard candidates that are present in this row or
            // column.
            for other_row in 0..9 {
                if let Some(solution) = self.board[other_row][col].solution {
                    if solution == candidate {
                        continue 'candidate_selection;
                    }
                }
            }
            for other_col in 0..9 {
                if let Some(solution) = self.board[row][other_col].solution {
                    if solution == candidate {
                        continue 'candidate_selection;
                    }
                }
            }

            candidates.insert(candidate);
        }

        candidates
    }

    /// Called when a solution for a cell is found in the preliminary candidate
    /// assignment phase. The solution is removed from the candidate list of all
    /// cells in the same row, column, and square, thus further narrowing down
    /// the search-space.
    fn found_solution(&mut self, row: usize, col: usize, solution: i8) {
        // We have a solution for this cell.
        let cell = &mut self.board[row][col];
        let block = &mut self.blocks[block_index(row, col)];
        cell.solution = Some(solution);
        cell.candidates.clear();
        block.solutions.insert(solution);

        // Remove candidates in this block, row, and column that are the same as
        // this solution.
        for other_row in 0..9 {
            self.board[other_row][col].candidates.remove(&solution);
        }

        for other_col in 0..9 {
            self.board[row][other_col].candidates.remove(&solution);
        }

        let block_row_start = (row / 3) * 3;
        let block_col_start = (col / 3) * 3;
        for block_row in block_row_start..block_row_start + 3 {
            for block_col in block_col_start..block_col_start + 3 {
                self.board[block_row][block_col].candidates.remove(&solution);
            }
        }
    }

    /// A brute-force, backtracking algorithm that attempts to guess solutions for cells as
    /// a function of previous guesses made for other cells.
    fn guess_solutions(&mut self) -> Option<Board> {
        let unsolved_cells = self.unsolved_cells();
        let mut cell_idx = 0;
        'cell_iteration: while cell_idx < unsolved_cells.len() {
            let (row, col) = unsolved_cells[cell_idx];
            let mut cand_idx = match self.board[row][col].candidate_idx {
                Some(idx) => idx,
                None => 0,
            };
            while cand_idx < self.board[row][col].candidates.len() {
                let candidate = *self.board[row][col].candidates
                    .iter()
                    .nth(cand_idx)
                    .unwrap();
                self.board[row][col].candidate = Some(candidate);
                // Make sure to increment candidate index *before* going to the
                // next cell so should we backtrack and end up here again, we
                // choose the next candidate instead of this one.
                cand_idx += 1;
                self.board[row][col].candidate_idx = Some(cand_idx);
                // If this candidate is good, go to the next cell.
                if self.can_choose_candidate(row, col, candidate) {
                    cell_idx += 1;
                    continue 'cell_iteration;
                }
            }

            // If we're here, it means we haven't found any eligible candidate for this
            // cell, so we need to backtrack. Reset candidate and its index so the next
            // time we're here we can retry all candidates again.
            self.board[row][col].candidate = None;
            self.board[row][col].candidate_idx = None;
            // If we're back at the first field after not finding any
            // candidates, it means there is no solution.
            if cell_idx == 0 {
                return None;
            }
            cell_idx -= 1;
        }

        self.use_final_candidates();

        Some(self.board.clone())
    }

    /// Returns a vector of (row, column) coordinates of the cells that are yet
    /// to be solved.
    fn unsolved_cells(&self) -> Vec<(usize, usize)> {
        let mut unsolved_cells = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                if self.board[row][col].solution.is_none() {
                    unsolved_cells.push((row, col));
                }
            }
        }
        unsolved_cells
    }

    /// Iterates over unsolved cells and makes their chosen candidate as their solution.
    fn use_final_candidates(&mut self) {
        for row in 0..9 {
            for col in 0..9 {
                let cell = &mut self.board[row][col];
                if cell.solution.is_none() {
                    if let Some(cand) = cell.candidate {
                        cell.solution = Some(cand);
                    } else {
                        println!("WARN: missing solution at {}:{}", row, col);
                    }
                }
            }
        }
    }

    /// Determines whether we can choose candidate for this cell based on
    /// previous candidate choices. Candidate is otherwise assumed to be correct
    /// based on other cells solved in its block, row, and column.
    fn can_choose_candidate(&self, row: usize, col: usize, candidate: i8) -> bool {
        // TODO: maybe we could use a reverse index to avoid all these iterations?
        for other_col in 0..col {
            let other_cell = &self.board[row][other_col];
            if other_cell.solution.is_none() {
                if let Some(other_cand) = other_cell.candidate {
                    if other_cand == candidate {
                        return false;
                    }
                }
            }
        }

        for other_row in 0..row {
            let other_cell = &self.board[other_row][col];
            if other_cell.solution.is_none() {
                if let Some(other_cand) = other_cell.candidate {
                    if other_cand == candidate {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Represents a 3x3 block of cells in a Sudoku board. This is used by the
/// solver to quickly verify that a candidate is not already solved in its
/// block.
#[derive(Debug, Eq, PartialEq)]
struct Block {
    // TODO: use BitSet or just a u16
    solutions: HashSet<i8>,
}

/// Partitions a Sudoku board into a vector of blocks.
fn make_blocks(board: &Board) -> [Block; 9] {
    // TODO remove unsafe code once Block is copyable (i.e. when switching to an
    // i16 bitmask for solutions)
    let mut blocks: [Block; 9] = unsafe {
        let mut blocks: [Block; 9] = std::mem::uninitialized();
        // Fill blocks vec. TODO more idiomatic way of doing this?
        for element in blocks.iter_mut() {
            let block = Block { solutions: HashSet::new() };
            // Overwrite element without running the destructor of the old value.
            std::ptr::write(element, block);
        }
        blocks
    };

    for (row_idx, row) in board.iter().enumerate() {
        for (col_idx, col) in row.iter().enumerate() {
            if let Some(num) = col.solution {
                let block_idx = block_index(row_idx, col_idx);
                assert!(block_idx < blocks.len());
                blocks[block_idx].solutions.insert(num);
            }
        }
    }

    blocks
}

/// Returns the index of a block (in a vector of nine blocks) to which the cell
/// at `row:col` belongs.
fn block_index(row: usize, col: usize) -> usize {
    let block_idx = row / 3 * 3 + col / 3;
    assert!(block_idx < 9);
    block_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_blocks() {
        let board = default_board();
        let blocks = make_blocks(&board);
        println!("{:#?}", blocks);

        assert_eq!(blocks, vec![
            Block { solutions: vec![5, 2, 7, 9].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![8, 3, 4, 5].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![5, 6, 2].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![4, 9, 1, 7].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![6, 4, 5, 7, 8, 2].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![7, 8, 1, 3].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![5, 4, 6].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![7, 8, 3, 1].iter().cloned().collect::<HashSet<i8>>(), },
            Block { solutions: vec![9, 6, 5, 4].iter().cloned().collect::<HashSet<i8>>(), },
        ]);
    }

    #[test]
    fn test_solver() {
        let board = default_board();
        let mut sudoku = Sudoku::new(board);
        if let Some(solved_board) = sudoku.solve() {
            for row in 0..9 {
                for col in 0..9 {
                    let solution = solved_board[row][col].solution;

                    // Check that this cell's solution is unique in its block.
                    let block_row_start = (row / 3) * 3;
                    let block_col_start = (col / 3) * 3;
                    for block_row in block_row_start..block_row_start + 3 {
                        for block_col in block_col_start..block_col_start + 3 {
                            if block_row == row && block_col == col {
                                continue;
                            }
                            assert_ne!(solution, solved_board[block_row][block_col].solution);
                        }
                    }

                    // Verify that solution is unique in its row.
                    for other_col in 0..9 {
                        if other_col != col {
                            assert_ne!(solution, solved_board[row][other_col].solution);
                        }
                    }

                    // Verify that solution is unique in its column.
                    for other_row in 0..9 {
                        if other_row != row {
                            assert_ne!(solution, solved_board[other_row][col].solution);
                        }
                    }
                }
            }
        } else {
            assert!(false);
        }
    }
}

fn solved(n: i8) -> Cell {
    Cell::solved(n)
}

fn unsolved() -> Cell {
    Cell::unsolved()
}

fn print_board(board: &Board) {
    let border = {
        let mut s = String::new();
        s.push('|');
        for _ in 0..35 {
            s.push('=');
        }
        s.push('|');
        s
    };
    let separator = {
        let mut s = String::new();
        s.push('|');
        for _ in 0..3 {
            for _ in 0..11 {
                s.push('-');
            }
            s.push('|');
        }
        s
    };

    let mut num_lines = 0;
    for row in board.iter() {
        if num_lines % 3 == 0 {
            println!("{}", border);
        } else {
            println!("{}", separator);
        }
        let mut line = String::from("|");
        for col in row.iter() {
            match col.solution {
                Some(solution) => {
                    line += &format!(" {} |", solution);
                },
                None => {
                    line += &String::from("   |");
                }
            }
        }
        println!("{}", line);
        num_lines += 1;
    }
    println!("{}", border);
}

fn default_board() -> Board {
    [
        [
            unsolved(), unsolved(), solved(5),
            unsolved(), unsolved(), solved(8),
            unsolved(), unsolved(), unsolved(),
        ],
        [
            unsolved(), solved(2), unsolved(),
            unsolved(), unsolved(), unsolved(),
            solved(5), unsolved(), unsolved(),
        ],
        [
            solved(7), solved(9), unsolved(),
            solved(3), solved(4), solved(5),
            solved(6), solved(2), unsolved(),
        ],

        [
            unsolved(), unsolved(), unsolved(),
            solved(6), unsolved(), solved(4),
            solved(7), solved(1), unsolved(),
        ],
        [
            unsolved(), solved(4), solved(9),
            solved(5), unsolved(), solved(7),
            solved(8), solved(3), unsolved(),
        ],
        [
            unsolved(), solved(1), solved(7),
            solved(8), unsolved(), solved(2),
            unsolved(), unsolved(), unsolved(),
        ],

        [
            unsolved(), solved(5), solved(4),
            solved(7), solved(8), solved(3),
            unsolved(), solved(9), solved(6),
        ],
        [
            unsolved(), unsolved(), solved(6),
            unsolved(), unsolved(), unsolved(),
            unsolved(), solved(5), unsolved(),
        ],
        [
            unsolved(), unsolved(), unsolved(),
            solved(1), unsolved(), unsolved(),
            solved(4), unsolved(), unsolved(),
        ],
    ]
}
