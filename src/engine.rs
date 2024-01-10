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

// mark square as mine
// if condradiction => it is safe

// if mine/safe is added at (x,y) is the system still consistent?
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
    // i/o args:

    //let mut constraints: Vec<Vec<Option<Constraint>>> = Vec::new();
    // monad -> do we know state of variable?
    //let mut mine: Vec<Vec<Option<bool>>> = Vec::new();

    match mine.index_2d_mut(x, y) {
        m @ None => *m = Some(set_mine),
        Some(m) => return set_mine == *m,
    };

    let mut valid = true;

    //*mine.index_2d_mut(x, y) = Some(set_mine);

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

    // let mut active_constraints: HashSet<(usize, usize)> = HashSet::new();

    // active_constraints.insert((initial_x, initial_y));
    
    let mut guess_made = true;
    while guess_made {
        guess_made = false;
        board.reset_write_head();
        {
            let mut border: HashSet<(usize, usize)> = HashSet::new();

            for y0 in 0..board.height {
                for x0 in 0..board.height {
                    // for each unkunknown
                    if let Some(&None) = mines.get_2d(x0, y0) {
                        for (x, y) in surrounding(x0, y0) {
                            if let Some(&Some(_)) = constraints.get_2d(x, y) {
                                border.insert((x0, y0));
                            }
                        }
                    }
                }
            }

            let mut border: Vec<(usize, usize)> = border.into_iter().collect();
            border.sort();
            let border_len = border.len();

            for (x_unknown, y_unknown) in border.iter().copied() {
                let Some(&None) = mines.get_2d(x_unknown, y_unknown) else {
                    continue;
                };
                for (x_constraint, y_constraint) in surrounding(x_unknown, y_unknown) {
                    if let Some(&Some(constraint)) = constraints.get_2d(x_constraint, y_constraint)
                    {
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

            for i in 0..border_len {
                board.reset_write_head();
                println!("{} / {}             ", i+1, border_len);
                if border.len() != 0 && mines.index_2d(border[0].0, border[0].1).is_none() {
                    let x = border[0].0;
                    let y = border[0].1;
                    // would it be inconsistent to place a mine here => safe
                    if !check_consistent(&mut constraints, &mut mines, 0, &border, true) {
                        make_guess(&mut board, &mut mines, &mut constraints, x, y, false);
                        guess_made = true;
                        //active_constraints.insert((x, y));
                    // would it be inconsistent if there was no mine here => safe
                    } else if !check_consistent(&mut constraints, &mut mines, 0, &border, false) {
                        make_guess(&mut board, &mut mines, &mut constraints, x, y, true);
                        guess_made = true;
                    }
                }
                let first = border.remove(0); // can be replaced with clever swap
                border.push(first);
            }

            //break;

            // fn check_consistent(
            //     constraints: &mut impl Index2D<Option<Constraint>>,
            //     mine: &mut impl Index2D<Option<bool>>,
            //     idx: usize,
            //     border: &[(usize, usize)],
            //     set_mine: bool,
            // ) -> bool {
        }
        std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
    }
    println!("{} squares remaining with unknown state", board.remaining);
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

/*fn play_simple(board: GameBoard) {
    let mut board = board;

    let mut state = vec![vec![Unknown; board.width]; board.height];

    let (initial_x, initial_y) = board.initial();
    state[initial_y][initial_x] = Element::from_query(board.query(initial_x, initial_y));

    let mut progress_was_made = true;
    while progress_was_made {
        progress_was_made = false;
        for y in 0..board.height {
            for x in 0..board.width {
                let Count(c) = state[y][x] else {
                    continue;
                };
                let mut mines = 0;
                let mut unknown = 0;

                for dx in [-1_i32, 0, 1] {
                    for dy in [-1_i32, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let x = x.wrapping_add(dx as usize);
                        let y = y.wrapping_add(dy as usize);
                        if x >= board.width || y >= board.height {
                            continue;
                        }
                        match state[y][x] {
                            Unknown => unknown += 1,
                            Mine => mines += 1,
                            Count(_) => revealed += 1,
                        }
                    }
                }
                if unknown == 0 {
                    continue;
                }

                let remaining_is_safe = c == mines;
                let remaining_is_mines = c == unknown + mines;
                if remaining_is_safe || remaining_is_mines {
                    for dx in [-1_i32, 0, 1] {
                        for dy in [-1_i32, 0, 1] {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let x = x.wrapping_add(dx as usize);
                            let y = y.wrapping_add(dy as usize);
                            if x >= board.width || y >= board.height {
                                continue;
                            }
                            if state[y][x] == Unknown {
                                state[y][x] =
                                    Element::from_query(board.test(x, y, remaining_is_mines));
                                progress_was_made = true;
                            }
                        }
                    }
                }
            }
        }
    }
}*/
