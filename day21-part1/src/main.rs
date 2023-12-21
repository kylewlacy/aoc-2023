use std::{collections::HashSet, io::Read as _};

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

    let map_rows = input
        .lines()
        .map(|line| line.chars().map(MapCell::try_from).collect())
        .collect::<eyre::Result<Vec<Vec<_>>>>()?;
    let start_pos = map_rows
        .iter()
        .enumerate()
        .find_map(|(i, row)| {
            row.iter().enumerate().find_map(|(j, cell)| match cell {
                MapCell::Start => Some(Position { row: i, col: j }),
                _ => None,
            })
        })
        .ok_or_eyre("starting position not found")?;
    let grid = Grid::new(&map_rows);

    let mut reachable_cells = HashSet::new();
    grid.reachable_cells(start_pos, 64, &mut reachable_cells);
    println!("{}", reachable_cells.len());

    Ok(())
}

struct Grid {
    rows: Vec<Vec<Cell>>,
}

impl Grid {
    fn new(map_rows: &[Vec<MapCell>]) -> Self {
        let rows: Vec<Vec<Cell>> = map_rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|cell| match cell {
                        MapCell::GardenPlot => Cell::GardenPlot,
                        MapCell::Rock => Cell::Rock,
                        MapCell::Start => Cell::GardenPlot,
                    })
                    .collect()
            })
            .collect();
        Self { rows }
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn offset(&self, position: Position, direction: Direction) -> Option<Position> {
        let new_position = match direction {
            Direction::North => Position {
                row: position.row.checked_sub(1)?,
                col: position.col,
            },
            Direction::South => Position {
                row: position.row + 1,
                col: position.col,
            },
            Direction::East => Position {
                row: position.row,
                col: position.col + 1,
            },
            Direction::West => Position {
                row: position.row,
                col: position.col.checked_sub(1)?,
            },
        };
        if new_position.row < self.num_rows() && new_position.col < self.num_cols() {
            Some(new_position)
        } else {
            None
        }
    }

    fn get(&self, position: Position) -> Option<Cell> {
        let row = self.rows.get(position.row)?;
        let cell = row.get(position.col)?;
        Some(*cell)
    }

    fn reachable_cells(
        &self,
        position: Position,
        distance: u64,
        positions: &mut HashSet<Position>,
    ) {
        match self.get(position) {
            Some(Cell::Rock) | None => {
                return;
            }
            Some(Cell::GardenPlot) => {}
        }

        if distance == 0 {
            positions.insert(position);
            return;
        }

        let next_positions = Direction::DIRECTIONS
            .iter()
            .filter_map(|direction| self.offset(position, *direction));
        for next_pos in next_positions {
            self.reachable_cells(next_pos, distance - 1, positions);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapCell {
    GardenPlot,
    Rock,
    Start,
}

impl TryFrom<char> for MapCell {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::GardenPlot),
            '#' => Ok(Self::Rock),
            'S' => Ok(Self::Start),
            other => {
                eyre::bail!("invalid map cell: {other:?}");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    GardenPlot,
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    const DIRECTIONS: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];
}
