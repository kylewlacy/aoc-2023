use std::io::Read as _;

use eyre::OptionExt;

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

    let mut grid = Grid::new(800, 800);
    let mut pos = grid.center();
    let instructions: Vec<Instruction> = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<_>>()?;

    for instruction in &instructions {
        dig(&mut grid, &mut pos, instruction);
    }

    let trench_volume: usize = grid
        .rows
        .iter()
        .map(|row| row.iter().filter(|cell| matches!(cell, Cell::Hole)).count())
        .sum();

    tracing::info!("{trench_volume}");

    dig_out_interior(&mut grid);

    let volume: usize = grid
        .rows
        .iter()
        .map(|row| row.iter().filter(|cell| matches!(cell, Cell::Hole)).count())
        .sum();

    tracing::info!("{volume}");

    Ok(())
}

fn dig(grid: &mut Grid, pos: &mut Position, instruction: &Instruction) {
    for _ in 0..instruction.distance {
        let cell = grid.get_mut(*pos).expect("position out of bounds");
        *cell = Cell::Hole;
        *pos = grid
            .move_position(*pos, instruction.direction, 1)
            .expect("moved out of bounds");
    }
}

fn dig_out_interior(grid: &mut Grid) {
    let mut should_dig = vec![vec![true; grid.num_cols()]; grid.num_rows()];

    for (i, row) in grid.rows.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            match cell {
                Cell::Ground => {}
                Cell::Hole => {
                    should_dig[i][j] = false;
                }
            }
        }
    }

    flood_fill((0, 0), &mut should_dig);

    // Uncomment to print flood fill map:
    // for row in &should_dig {
    //     for cell in row {
    //         if *cell {
    //             print!("#");
    //         } else {
    //             print!(".");
    //         }
    //     }
    //     println!();
    // }

    for (i, row) in should_dig.iter().enumerate() {
        for (j, should_dig_cell) in row.iter().enumerate() {
            if *should_dig_cell {
                grid.rows[i][j] = Cell::Hole;
            }
        }
    }
}

fn flood_fill(pos: (isize, isize), should_dig: &mut [Vec<bool>]) {
    let num_rows = should_dig.len();
    let num_cols = should_dig.get(0).map(|row| row.len()).unwrap_or(0);

    let Ok(i) = pos.0.try_into() else {
        return;
    };
    let Ok(j) = pos.1.try_into() else {
        return;
    };
    let (i, j): (usize, usize) = (i, j);
    if i >= num_rows || j >= num_cols {
        return;
    }

    if !should_dig[i][j] {
        return;
    }
    should_dig[i][j] = false;

    flood_fill((pos.0 + 1, pos.1), should_dig);
    // flood_fill((pos.0 - 1, pos.1), should_dig);
    flood_fill((pos.0, pos.1 + 1), should_dig);
    flood_fill((pos.0, pos.1 - 1), should_dig);
}

struct Grid {
    rows: Vec<Vec<Cell>>,
}

impl Grid {
    fn new(rows: usize, cols: usize) -> Self {
        let rows = vec![vec![Cell::Ground; cols]; rows];

        Self { rows }
    }

    fn center(&self) -> Position {
        Position {
            row: self.num_rows() as isize / 2,
            col: self.num_cols() as isize / 2,
        }
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn move_position(
        &self,
        position: Position,
        direction: Direction,
        distance: isize,
    ) -> Option<Position> {
        let new_position = match direction {
            Direction::Up => Position {
                row: position.row - distance,
                col: position.col,
            },
            Direction::Down => Position {
                row: position.row + distance,
                col: position.col,
            },
            Direction::Left => Position {
                row: position.row,
                col: position.col - distance,
            },
            Direction::Right => Position {
                row: position.row,
                col: position.col + distance,
            },
        };
        let new_row: usize = new_position.row.try_into().ok()?;
        let new_col: usize = new_position.col.try_into().ok()?;

        if new_row >= self.num_rows() || new_col >= self.num_cols() {
            return None;
        }

        Some(new_position)
    }

    fn get_mut(&mut self, position: Position) -> Option<&mut Cell> {
        let i: usize = position.row.try_into().ok()?;
        let j: usize = position.col.try_into().ok()?;
        let row = self.rows.get_mut(i)?;
        let cell = row.get_mut(j)?;
        Some(cell)
    }
}

#[derive(Debug, Clone, Copy)]
enum Cell {
    Ground,
    Hole,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    direction: Direction,
    distance: isize,
}

impl std::str::FromStr for Instruction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();

        let direction = parts.next().ok_or_eyre("invalid instruction")?;
        let direction = direction.parse()?;

        let distance = parts.next().ok_or_eyre("invalid instruction")?;
        let distance = distance.parse()?;

        let _color_code = parts.next();

        Ok(Self {
            direction,
            distance,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl std::str::FromStr for Direction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Self::Up),
            "D" => Ok(Self::Down),
            "L" => Ok(Self::Left),
            "R" => Ok(Self::Right),
            other => {
                eyre::bail!("invalid direction: {other:?}");
            }
        }
    }
}
