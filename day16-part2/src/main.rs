use std::io::Read as _;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .without_time()
        .init();
    color_eyre::install()?;

    let mut stdin = std::io::stdin().lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input)?;

    let mut grid = Grid::parse_new(&input)?;
    let num_rows = grid.num_rows();
    let num_cols = grid.num_cols();
    let top_edge_starts = (0..num_cols).map(|col| (Position { row: 0, col }, Direction::Down));
    let right_edge_starts = (0..num_rows).map(|row| {
        (
            Position {
                row,
                col: num_cols - 1,
            },
            Direction::Left,
        )
    });
    let bottom_edge_starts = (0..num_cols).map(|col| {
        (
            Position {
                row: num_rows - 1,
                col,
            },
            Direction::Up,
        )
    });
    let left_edge_starts = (0..num_rows).map(|row| (Position { row, col: 0 }, Direction::Right));
    let starts = top_edge_starts
        .chain(right_edge_starts)
        .chain(bottom_edge_starts)
        .chain(left_edge_starts);

    let most_energy = starts
        .map(|(pos, dir)| {
            let mut grid = grid.clone();
            grid.energize(pos, dir);
            grid.num_energized()
        })
        .max()
        .unwrap_or(0);

    println!("{most_energy}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Grid {
    rows: Vec<Vec<GridCell>>,
}

impl Grid {
    fn parse_new(s: &str) -> eyre::Result<Self> {
        let rows = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| {
                        let contraption = Contraption::try_from(c)?;
                        eyre::Ok(GridCell::new(contraption))
                    })
                    .collect::<eyre::Result<Vec<_>>>()
            })
            .collect::<eyre::Result<Vec<Vec<_>>>>()?;
        Ok(Self { rows })
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn num_energized(&self) -> usize {
        self.rows
            .iter()
            .map(|row| {
                row.iter()
                    .filter(|row| row.energization.is_energized())
                    .count()
            })
            .sum()
    }

    fn energize(&mut self, position: Position, direction: Direction) {
        let next_directions = self.rows[position.row][position.col].energize(direction);
        for next_direction in &next_directions {
            let Some(next_pos) = self.move_position(position, *next_direction) else {
                continue;
            };

            self.energize(next_pos, *next_direction);
        }
    }

    fn move_position(&self, position: Position, direction: Direction) -> Option<Position> {
        let next_position = match direction {
            Direction::Up => Position {
                row: position.row.checked_sub(1)?,
                col: position.col,
            },
            Direction::Right => Position {
                row: position.row,
                col: position.col + 1,
            },
            Direction::Down => Position {
                row: position.row + 1,
                col: position.col,
            },
            Direction::Left => Position {
                row: position.row,
                col: position.col.checked_sub(1)?,
            },
        };

        if next_position.row < self.num_rows() && next_position.col < self.num_cols() {
            Some(next_position)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GridCell {
    contraption: Contraption,
    energization: Energization,
}

impl GridCell {
    fn new(contraption: Contraption) -> Self {
        GridCell {
            contraption,
            energization: Energization::default(),
        }
    }

    fn energize(&mut self, direction: Direction) -> Vec<Direction> {
        // Don't send out any beams if we've already energized this cell from
        // this direction.
        if !self.energization.energize(direction) {
            return vec![];
        }

        match (self.contraption, direction) {
            (Contraption::Empty, direction) => vec![direction],
            (Contraption::ForwardMirror, Direction::Up) => vec![Direction::Right],
            (Contraption::ForwardMirror, Direction::Right) => vec![Direction::Up],
            (Contraption::ForwardMirror, Direction::Down) => vec![Direction::Left],
            (Contraption::ForwardMirror, Direction::Left) => vec![Direction::Down],
            (Contraption::BackwardMirror, Direction::Up) => vec![Direction::Left],
            (Contraption::BackwardMirror, Direction::Right) => vec![Direction::Down],
            (Contraption::BackwardMirror, Direction::Down) => vec![Direction::Right],
            (Contraption::BackwardMirror, Direction::Left) => vec![Direction::Up],
            (Contraption::VerticalSplitter, direction @ (Direction::Up | Direction::Down)) => {
                vec![direction]
            }
            (Contraption::VerticalSplitter, Direction::Right | Direction::Left) => {
                vec![Direction::Up, Direction::Down]
            }
            (Contraption::HorizontalSplitter, Direction::Up | Direction::Down) => {
                vec![Direction::Left, Direction::Right]
            }
            (Contraption::HorizontalSplitter, direction @ (Direction::Right | Direction::Left)) => {
                vec![direction]
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Contraption {
    Empty,
    ForwardMirror,
    BackwardMirror,
    VerticalSplitter,
    HorizontalSplitter,
}

impl TryFrom<char> for Contraption {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            '/' => Ok(Self::ForwardMirror),
            '\\' => Ok(Self::BackwardMirror),
            '|' => Ok(Self::VerticalSplitter),
            '-' => Ok(Self::HorizontalSplitter),
            other => {
                eyre::bail!("invalid contraption: {other:?}");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Energization {
    upward: bool,
    rightward: bool,
    downward: bool,
    leftward: bool,
}

impl Energization {
    fn energize(&mut self, direction: Direction) -> bool {
        let energization = match direction {
            Direction::Up => &mut self.upward,
            Direction::Right => &mut self.rightward,
            Direction::Down => &mut self.downward,
            Direction::Left => &mut self.leftward,
        };
        let is_already_energized = std::mem::replace(energization, true);
        !is_already_energized
    }

    fn is_energized(&self) -> bool {
        self.upward || self.rightward || self.downward || self.leftward
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    row: usize,
    col: usize,
}
