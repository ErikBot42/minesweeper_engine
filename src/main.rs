use std::fmt::Write;
struct GameBoard {
    mine: Vec<Vec<bool>>,
    count: Vec<Vec<u8>>,
    revealed: Vec<Vec<bool>>,
    width: usize,
    height: usize,
    remaining: usize,
    display: bool,
}
impl GameBoard {
    fn new(width: usize, height: usize, rng: Prng, display: bool) -> Self {
        let mut rng = rng;
        let mines: Vec<Vec<bool>> = (0..height)
            .map(|_| (0..width).map(|_| rng.next() & 0b11111 <= 6).collect())
            .collect();
        let revealed: Vec<Vec<bool>> = vec![vec![false; width]; height];

        let counts = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let mut count = 0;
                        for (x, y) in surrounding(x, y) {
                            count += mines
                                .get(y)
                                .map(|row| row.get(x))
                                .flatten()
                                .copied()
                                .unwrap_or(false) as u8;
                        }
                        count
                    })
                    .collect()
            })
            .collect();
        if display {
            let mut buffer = String::new();
            write!(&mut buffer, "\x1b[0;0H").unwrap();
            write!(&mut buffer, "+-").unwrap();
            for _ in 0..width {
                write!(&mut buffer, "--").unwrap();
            }
            write!(&mut buffer, "+\n").unwrap();

            for _ in 0..height {
                write!(&mut buffer, "| ").unwrap();
                for _ in 0..width {
                    write!(&mut buffer, "  ").unwrap();
                }
                write!(&mut buffer, "|\n").unwrap();
            }

            write!(&mut buffer, "+-").unwrap();
            for _ in 0..width {
                write!(&mut buffer, "--").unwrap();
            }
            write!(&mut buffer, "+\n").unwrap();

            print!("{buffer}");
        }
        Self {
            mine: mines,
            count: counts,
            revealed,
            width,
            height,
            remaining: width * height,
            display,
        }
    }
    fn query(&mut self, x: usize, y: usize) -> Option<u8> {
        //std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
        if !self.revealed[y][x] {
            self.remaining -= 1;
            self.revealed[y][x] = true;
        }
        self.display(x, y);
        (!self.mine[y][x]).then_some(self.count[y][x])
    }
    fn test(&mut self, x: usize, y: usize, mine: bool) -> Option<u8> {
        assert_eq!(
            self.mine[y][x], mine,
            "wrong guess at {x}, {y}, real: {}, guess: {}",
            self.mine[y][x], mine
        );
        self.query(x, y)
    }
    fn count_to_color(count: u8) -> Color {
        match count {
            1 => Color(221, 208, 122),
            2 => Color(224, 177, 89),
            3 => Color(226, 161, 70),
            4 => Color(229, 129, 48),
            5 => Color(232, 102, 32),
            6 => Color(234, 69, 18),
            7 => Color(247, 25, 4),
            8 => Color(255, 0, 0),
            _ => Color(255, 255, 255),
        }
    }
    fn display(&self, x: usize, y: usize) {
        if self.display {
            let mut buffer = String::new();
            let mut color = Color(255, 255, 255);
            write!(&mut buffer, "\x1b[{};{}H", y + 2, x * 2 + 2).unwrap();
            if self.revealed[y][x] {
                if self.mine[y][x] {
                    color.set(&mut buffer, Color(0, 255, 0)).unwrap();
                    write!(&mut buffer, "* ").unwrap();
                } else {
                    if self.count[y][x] == 0 {
                        color.set(&mut buffer, Color(255, 255, 255)).unwrap();
                        write!(&mut buffer, "_ ").unwrap();
                    } else {
                        color
                            .set(&mut buffer, Self::count_to_color(self.count[y][x]))
                            .unwrap();
                        write!(&mut buffer, "{} ", self.count[y][x]).unwrap();
                    }
                }
            }
            color.set(&mut buffer, Color(255, 255, 255)).unwrap();
            print!("{buffer}\n");
            self.reset_write_head();
        }
    }
    fn draw(&self, x: usize, y: usize, s: String) {
        if self.display {
            println!("\x1b[{};{}H{s}", y + 2, x * 2 + 2);
            self.reset_write_head();
        }
    }
    fn reset_write_head(&self) {
        if self.display {
            println!("\x1b[{};{}H", self.height + 2, 0);
        }
    }

    fn forall(&self) -> impl Iterator<Item = (usize, usize)> {
        let height = self.height;
        let width = self.width;
        (0..height).flat_map(move |y| (0..width).map(move |x| (x, y)))
    }

    fn initial(&self) -> (usize, usize) {
        for (x, y) in self.forall().skip(self.width) {
            if self.count[y][x] == 0 && !self.mine[y][x] {
                return (x, y);
            }
        }
        panic!("board has no zero counts");
    }
}

trait Index2D<T> {
    fn get_2d(&self, x: usize, y: usize) -> Option<&T>;
    fn get_2d_mut(&mut self, x: usize, y: usize) -> Option<&mut T>;

    fn index_2d(&self, x: usize, y: usize) -> &T {
        self.get_2d(x, y).unwrap()
    }
    fn index_2d_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.get_2d_mut(x, y).unwrap()
    }
}
impl<T> Index2D<T> for [Vec<T>] {
    fn get_2d(&self, x: usize, y: usize) -> Option<&T> {
        self.get(y).map(|row| row.get(x)).flatten()
    }

    fn get_2d_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.get_mut(y).map(|row| row.get_mut(x)).flatten()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Color(u8, u8, u8);
impl Color {
    //println!("\x1b[31;1;4mHello\x1b[0m");
    fn write<T: std::fmt::Write>(&self, mut buffer: T) -> std::fmt::Result {
        write!(buffer, "\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
    }
    fn set<T: std::fmt::Write>(&mut self, buffer: T, new: Color) -> std::fmt::Result {
        if *self == new {
            Ok(())
        } else {
            *self = new;
            self.write(buffer)
        }
    }
}

struct Prng(pub u64);

impl Prng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
#[rustfmt::skip]
fn surrounding(x: usize, y: usize) -> [(usize, usize); 8] {
    [
        (x.wrapping_add(1), y.wrapping_add(1)),
        (x,                 y.wrapping_add(1)),
        (x.wrapping_sub(1), y.wrapping_add(1)),

        (x.wrapping_add(1), y                ),
        //(x,                 y                ),
        (x.wrapping_sub(1), y                ),

        (x.wrapping_add(1), y.wrapping_sub(1)),
        (x,                 y.wrapping_sub(1)),
        (x.wrapping_sub(1), y.wrapping_sub(1)),
    ]
}

mod engine;
use engine::play;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let board = GameBoard::new(230, 125, Prng(23841421), true);
    play(board);
}
