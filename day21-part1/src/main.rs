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
                MapCell::Start => Some(Position { row: i, col: j }),
                _ => None,
            })
        })
        .ok_or_eyre("starting position not found")?;
    let grid = Grid::new(&map_rows);

    let mut reachable_cells = HashMap::new();
    let mut debug_grid: Vec<Vec<_>> = map_rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    MapCell::GardenPlot => ' ',
                    MapCell::Rock => '#',
                    MapCell::Start => 'S',
                })
                .collect()
        })
        .collect();
    find_reachable_cells(
        &grid,
        start_pos,
        0,
        64,
        &mut reachable_cells,
        &mut debug_grid,
    );
    debug_grid[start_pos.row][start_pos.col] = 'S';

    // Uncomment to print debug grid:
    // println!(
    //     "{}",
    //     debug_grid
    //         .into_iter()
    //         .map(|s| s.into_iter().collect::<String>())
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // );

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
}

fn find_reachable_cells(
    grid: &Grid,
    position: Position,
    current_distance: u64,
    max_distance: u64,
    reachable: &mut HashMap<Position, u64>,
    debug_grid: &mut [Vec<char>],
) {
    let already_reached = reachable
        .get(&position)
        .is_some_and(|d| *d <= current_distance);
    if already_reached {
        tracing::debug!(%position, current_distance, max_distance, "already reachable");
        return;
    }

    match grid.get(position) {
        Some(Cell::Rock) | None => {
            tracing::debug!(%position, current_distance, max_distance, "obstacle");
            return;
        }
        Some(Cell::GardenPlot) => {}
    }

    if current_distance == max_distance || current_distance % 2 == 0 {
        tracing::debug!(%position, current_distance, max_distance, "reached");
        reachable
            .entry(position)
            .and_modify(|e| *e = std::cmp::min(*e, current_distance))
            .or_insert(current_distance);
        debug_grid[position.row][position.col] = '_';
    }

    if current_distance == max_distance {
        return;
    }

    let next_positions = Direction::DIRECTIONS
        .iter()
        .filter_map(|direction| grid.offset(position, *direction));
    for next_pos in next_positions {
        find_reachable_cells(
            grid,
            next_pos,
            current_distance + 1,
            max_distance,
            reachable,
            debug_grid,
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
    row: usize,
    col: usize,
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
