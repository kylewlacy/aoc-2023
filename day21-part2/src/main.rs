// Thanks to my wife for a hint on solving this one!

use std::{collections::HashMap, io::Read as _};

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
                MapCell::Start => Some(Position {
                    row: i as isize,
                    col: j as isize,
                }),
                _ => None,
            })
        })
        .ok_or_eyre("starting position not found")?;
    let grid = Grid::new(&map_rows);

    let mut reachable_cells = HashMap::new();
    find_reachable_cells(&grid, start_pos, 0, 26501365, &mut reachable_cells);

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

    fn get(&self, position: Position) -> Cell {
        let row = &self.rows[position.row.rem_euclid(self.rows.len() as isize) as usize];
        let cell = row[position.col.rem_euclid(row.len() as isize) as usize];
        cell
    }
}

fn find_reachable_cells(
    grid: &Grid,
    position: Position,
    current_distance: u64,
    max_distance: u64,
    reachable: &mut HashMap<Position, u64>,
) {
    let already_reached = reachable
        .get(&position)
        .is_some_and(|d| *d <= current_distance);
    if already_reached {
        tracing::debug!(%position, current_distance, max_distance, "already reachable");
        return;
    }

    match grid.get(position) {
        Cell::Rock => {
            tracing::debug!(%position, current_distance, max_distance, "obstacle");
            return;
        }
        Cell::GardenPlot => {}
    }

    if current_distance == max_distance || current_distance % 2 == 0 {
        tracing::debug!(%position, current_distance, max_distance, "reached");
        reachable
            .entry(position)
            .and_modify(|e| *e = std::cmp::min(*e, current_distance))
            .or_insert(current_distance);
    }

    if current_distance == max_distance {
        return;
    }

    let next_positions = Direction::DIRECTIONS
        .iter()
        .map(|direction| position.offset(*direction));
    for next_pos in next_positions {
        find_reachable_cells(
            grid,
            next_pos,
            current_distance + 1,
            max_distance,
            reachable,
        );
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
    row: isize,
    col: isize,
}

impl Position {
    fn offset(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => Self {
                row: self.row - 1,
                col: self.col,
            },
            Direction::South => Self {
                row: self.row + 1,
                col: self.col,
            },
            Direction::East => Self {
                row: self.row,
                col: self.col + 1,
            },
            Direction::West => Self {
                row: self.row,
                col: self.col - 1,
            },
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
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
