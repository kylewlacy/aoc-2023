use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    io::Read as _,
};

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

    let grid = Grid::parse(&input)?;
    let start = grid.start()?;

    let mut loop_distances: HashMap<Position, usize> = HashMap::new();
    loop_distances.insert(start, 0);

    let mut steps = grid
        .connections(start)
        .into_iter()
        .map(|pos| (start, pos, 1))
        .collect::<VecDeque<_>>();
    tracing::debug!(?steps, "starting");
    while let Some((prev_position, position, distance)) = steps.pop_front() {
        match loop_distances.entry(position) {
            Entry::Occupied(mut entry) => {
                let min_steps = std::cmp::min(distance, *entry.get());
                entry.insert(min_steps);
            }
            Entry::Vacant(entry) => {
                entry.insert(distance);
            }
        }

        let next_positions = grid.connections(position);
        tracing::debug!(?next_positions, ?position, "next positions");
        let mut next_positions = next_positions
            .into_iter()
            .filter(|pos| *pos != prev_position);
        let next_position = next_positions.next().ok_or_eyre("no next position")?;
        eyre::ensure!(
            next_positions.next().is_none(),
            "expected there to be only one connection"
        );

        if grid.get(next_position) != Some(Cell::Start) {
            steps.push_back((position, next_position, distance + 1));
        }
    }

    let max_distance = loop_distances.values().max().copied().unwrap_or_default();

    println!("{max_distance}");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Vertical,
    Horizontal,
    NorthEastBend,
    NorthWestBend,
    SouthWestBend,
    SouthEastBend,
    Ground,
    Start,
}

impl TryFrom<char> for Cell {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '|' => Ok(Self::Vertical),
            '-' => Ok(Self::Horizontal),
            'L' => Ok(Self::NorthEastBend),
            'J' => Ok(Self::NorthWestBend),
            '7' => Ok(Self::SouthWestBend),
            'F' => Ok(Self::SouthEastBend),
            '.' => Ok(Self::Ground),
            'S' => Ok(Self::Start),
            other => {
                eyre::bail!("invalid cell: {other:?}");
            }
        }
    }
}

struct Grid {
    rows: Vec<Vec<Cell>>,
}

impl Grid {
    fn parse(s: &str) -> eyre::Result<Self> {
        let mut rows = vec![];
        for line in s.lines() {
            let row = line
                .chars()
                .map(|c| c.try_into())
                .collect::<eyre::Result<Vec<Cell>>>()?;
            rows.push(row);
        }

        Ok(Self { rows })
    }

    fn cells(&self) -> impl Iterator<Item = (Position, Cell)> + '_ {
        self.rows.iter().enumerate().flat_map(|(i, row)| {
            row.iter().copied().enumerate().map(move |(j, cell)| {
                (
                    Position {
                        row: i.try_into().unwrap(),
                        col: j.try_into().unwrap(),
                    },
                    cell,
                )
            })
        })
    }

    fn start(&self) -> eyre::Result<Position> {
        let start = self.cells().find_map(|(pos, cell)| match cell {
            Cell::Start => Some(pos),
            _ => None,
        });

        start.ok_or_eyre("start not found")
    }

    fn get(&self, position: Position) -> Option<Cell> {
        let row: usize = position.row.try_into().ok()?;
        let col: usize = position.col.try_into().ok()?;
        let cell = self.rows.get(row)?.get(col)?;
        Some(*cell)
    }

    fn connections(&self, pos: Position) -> Vec<Position> {
        let cell = self.get(pos).expect("position out of bounds");

        match cell {
            Cell::Vertical => vec![pos + Position::UP, pos + Position::DOWN],
            Cell::Horizontal => vec![pos + Position::LEFT, pos + Position::RIGHT],
            Cell::NorthEastBend => vec![pos + Position::UP, pos + Position::RIGHT],
            Cell::NorthWestBend => vec![pos + Position::UP, pos + Position::LEFT],
            Cell::SouthWestBend => vec![pos + Position::DOWN, pos + Position::LEFT],
            Cell::SouthEastBend => vec![pos + Position::DOWN, pos + Position::RIGHT],
            Cell::Ground => vec![],
            Cell::Start => {
                let mut neighbors = vec![
                    pos + Position::UP,
                    pos + Position::RIGHT,
                    pos + Position::DOWN,
                    pos + Position::LEFT,
                ];
                neighbors.retain(|neighbor| self.get(*neighbor).is_some());
                neighbors.retain(|neighbor| self.connections(*neighbor).contains(&pos));
                neighbors
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: i32,
    col: i32,
}

impl Position {
    const UP: Self = Self { row: -1, col: 0 };
    const DOWN: Self = Self { row: 1, col: 0 };
    const LEFT: Self = Self { row: 0, col: -1 };
    const RIGHT: Self = Self { row: 0, col: 1 };
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row + rhs.row,
            col: self.col + rhs.col,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row - rhs.row,
            col: self.col - rhs.col,
        }
    }
}
