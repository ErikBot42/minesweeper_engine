use super::{surrounding, GameBoard, Index2D};

#[derive(Copy, Clone, Debug)]
struct Constraint {
    count: i8,
    mines: i8,
    unknown: i8, // remaining unknown
}
impl Constraint {
    fn new(count: i8, mines: i8, unknown: i8) -> Self {
        Self {
            count,
            mines,
            unknown,
        }
    }
    fn valid(self) -> bool {
        let Constraint {
            count,
            mines,
            unknown,
        } = self;
        mines <= count && count <= mines + unknown
    }
    fn remaining_safe(self) -> bool {
        self.count == self.mines
    }
    fn remaining_mines(self) -> bool {
        self.count == self.mines + self.unknown
    }
    fn add_mine(&mut self, delta: i8) {
        self.mines += delta;
        self.unknown -= delta;
    }
    fn add_safe(&mut self, delta: i8) {
        self.unknown -= delta;
    }
}

/// if a mine/safe is added at (x,y) is the system still consistent?
fn check_consistent(
    constraints: &mut Vec<Vec<Option<Constraint>>>,
    mine: &mut Vec<Vec<Option<bool>>>,
    idx: usize,
    border: &[(usize, usize)],
    set_mine: bool,
) -> bool {
    let Some(&(x, y)) = border.get(idx) else {
        return true;
    };

    match mine.index_2d_mut(x, y) {
        m @ None => *m = Some(set_mine),
        Some(m) => return set_mine == *m,
    };

    let mut valid = true;

    for (x, y) in surrounding(x, y) {
        if let Some(Some(constraint)) = constraints.get_2d_mut(x, y) {
            if set_mine {
                constraint.add_mine(1);
            } else {
                constraint.add_safe(1);
            }
            valid &= constraint.valid();
        }
    }

    valid = valid
        && (check_consistent(constraints, mine, idx + 1, border, false)
            || check_consistent(constraints, mine, idx + 1, border, true));

    *mine.index_2d_mut(x, y) = None;
    for (x, y) in surrounding(x, y) {
        if let Some(Some(constraint)) = constraints.get_2d_mut(x, y) {
            if set_mine {
                constraint.add_mine(-1);
            } else {
                constraint.add_safe(-1);
            }
            valid &= constraint.valid();
        }
    }
    return valid;
}

pub fn play(board: GameBoard) {
    let mut board = board;
    let (initial_x, initial_y) = board.initial();

    let mut constraints: Vec<Vec<Option<Constraint>>> = vec![vec![None; board.width]; board.height];
    let mut mines: Vec<Vec<Option<bool>>> = vec![vec![None; board.width]; board.height];

    let expected_mine = false;
    make_guess(
        &mut board,
        &mut mines,
        &mut constraints,
        initial_x,
        initial_y,
        expected_mine,
    );

    use std::collections::HashSet;

    let mut guess_made = true;
    while guess_made {
        guess_made = false;
        board.reset_write_head();

        if board.display {
            println!("fast pass             ");
        }
        for (x_unknown, y_unknown) in board.forall() {
            let Some(&None) = mines.get_2d(x_unknown, y_unknown) else {
                continue;
            };
            for (x_constraint, y_constraint) in surrounding(x_unknown, y_unknown) {
                if let Some(&Some(constraint)) = constraints.get_2d(x_constraint, y_constraint) {
                    if constraint.remaining_safe() {
                        make_guess(
                            &mut board,
                            &mut mines,
                            &mut constraints,
                            x_unknown,
                            y_unknown,
                            false,
                        );
                        guess_made = true;
                        break;
                    } else if constraint.remaining_mines() {
                        make_guess(
                            &mut board,
                            &mut mines,
                            &mut constraints,
                            x_unknown,
                            y_unknown,
                            true,
                        );
                        guess_made = true;
                        break;
                    } else {
                    }
                }
            }
        }
        board.reset_write_head();

        if guess_made {
            continue;
        }

        let borders = {
            let mut iter = board.forall();

            let mut in_any_border = vec![vec![false; board.width]; board.height];

            let mut borders = Vec::new();

            while let Some((x0, y0)) = iter.next() {
                if in_any_border[y0][x0] {
                    continue;
                }
                // start at unknown
                let Some(&None) = mines.get_2d(x0, y0) else {
                    continue;
                };
                let mut any_surrounding_constraint = false;
                for (x, y) in surrounding(x0, y0) {
                    if let Some(&Some(_)) = constraints.get_2d(x, y) {
                        any_surrounding_constraint = true;
                    }
                }
                if !any_surrounding_constraint {
                    continue;
                }

                // stack, border contains unknown variables
                let mut stack = Vec::new();
                let mut border: HashSet<(usize, usize)> = HashSet::new();
                stack.push((x0, y0));

                while let Some((x, y)) = stack.pop() {
                    if border.contains(&(x, y)) {
                        continue;
                    }
                    border.insert((x, y));
                    for (constraint_x, constraint_y) in surrounding(x, y) {
                        if matches!(
                            constraints.get_2d(constraint_x, constraint_y),
                            Some(&None) | None
                        ) {
                            continue;
                        }
                        for (unknown_x, unknown_y) in surrounding(constraint_x, constraint_y) {
                            if matches!(mines.get_2d(unknown_x, unknown_y), Some(&Some(_)) | None) {
                                continue;
                            }
                            stack.push((unknown_x, unknown_y));
                        }
                    }
                }
                let mut border: Vec<(usize, usize)> = border.into_iter().collect();
                border.sort();
                for (x, y) in border.iter().copied() {
                    in_any_border[y][x] = true;
                }
                borders.push(border);
            }

            borders.sort_unstable_by_key(|b| b.len());
            borders
        };

        for (i, mut border) in borders.into_iter().enumerate() {
            let c = [
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            ][i % 26];

            for b in border.iter().copied() {
                board.draw(b.0, b.1, format!("{c}"));
            }
            let border_len = border.len();

            for i in 0..border_len {
                if board.display {
                    board.reset_write_head();
                    println!("{} / {}             ", i + 1, border_len);
                }
                if border.len() != 0 && mines.index_2d(border[0].0, border[0].1).is_none() {
                    let x = border[0].0;
                    let y = border[0].1;
                    // would it be inconsistent to place a mine here => safe
                    if !check_consistent(&mut constraints, &mut mines, 0, &border, true) {
                        make_guess(&mut board, &mut mines, &mut constraints, x, y, false);
                        guess_made = true;

                    // would it be inconsistent if there was no mine here => safe
                    } else if !check_consistent(&mut constraints, &mut mines, 0, &border, false) {
                        make_guess(&mut board, &mut mines, &mut constraints, x, y, true);
                        guess_made = true;
                    }
                }
                let first = border.remove(0); // can be replaced with clever swap
                border.push(first);
            }
            if guess_made {
                break;
            }
        }
    }
    println!(
        "{} squares ({}%) remaining with unknown state",
        board.remaining,
        board.remaining as f64 / (board.width as f64 * board.height as f64)
    );
}

fn make_guess(
    board: &mut GameBoard,
    mines: &mut [Vec<Option<bool>>],
    constraints: &mut [Vec<Option<Constraint>>],
    x: usize,
    y: usize,
    expected_mine: bool,
) {
    let query = board.test(x, y, expected_mine);
    match query {
        Some(c) => {
            let count = c as i8;
            let mut mine_count = 0;
            let mut unknown = 0;
            for (x, y) in surrounding(x, y) {
                if let Some(&m) = mines.get_2d(x, y) {
                    match m {
                        Some(true) => mine_count += 1,
                        Some(false) => (),
                        None => unknown += 1,
                    }
                }
            }
            *mines.index_2d_mut(x, y) = Some(false);
            *constraints.index_2d_mut(x, y) = Some(Constraint::new(count, mine_count, unknown));
        }
        None => {
            *mines.index_2d_mut(x, y) = Some(true);
        }
    }
    // update surrounding constraints
    for (x, y) in surrounding(x, y) {
        if let Some(Some(constraint)) = constraints.get_2d_mut(x, y) {
            if expected_mine {
                constraint.add_mine(1);
            } else {
                constraint.add_safe(1);
            }
        }
    }
}
