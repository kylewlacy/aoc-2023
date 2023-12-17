use std::io::Read as _;

use eyre::OptionExt as _;
use pathfinding::directed::dijkstra::dijkstra;

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

    let grid: Grid = input.parse()?;
    let crucible = Crucible::new();

    let path = dijkstra(
        &crucible,
        |crucible| {
            let candidates = crucible.move_candidates(&grid);
            candidates.into_iter().map(|candidate| {
                let heat_loss = candidate.heat_loss_at_position(&grid);
                (candidate, heat_loss)
            })
        },
        |crucible| crucible.is_finished(&grid),
    );
    let (_path, total_heat_loss) = path.ok_or_eyre("no path found")?;
    println!("{total_heat_loss}");

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Grid {
    rows: Vec<Vec<u32>>,
}

impl Grid {
    fn get(&self, pos: Position) -> Option<u32> {
        let row = self.rows.get(pos.row)?;
        let cell = row.get(pos.col)?;
        Some(*cell)
    }

    fn move_position(&self, pos: Position, dir: Direction) -> Option<Position> {
        let new_pos = match dir {
            Direction::North => Position {
                row: pos.row.checked_sub(1)?,
                col: pos.col,
            },
            Direction::South => Position {
                row: pos.row + 1,
                col: pos.col,
            },
            Direction::East => Position {
                row: pos.row,
                col: pos.col + 1,
            },
            Direction::West => Position {
                row: pos.row,
                col: pos.col.checked_sub(1)?,
            },
        };

        if self.get(new_pos).is_none() {
            return None;
        }

        Some(new_pos)
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn end(&self) -> Position {
        Position {
            row: self.num_rows().saturating_sub(1),
            col: self.num_cols().saturating_sub(1),
        }
    }
}

impl std::str::FromStr for Grid {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| c.to_digit(10).ok_or_eyre("invalid digit"))
                    .collect::<eyre::Result<Vec<_>>>()
            })
            .collect::<eyre::Result<Vec<Vec<_>>>>()?;
        Ok(Self { rows })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    const DIRECTIONS: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];

    fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Crucible {
    position: Position,
    direction_history: Vec<Direction>,
}

impl Crucible {
    fn new() -> Self {
        Self {
            position: Position { row: 0, col: 0 },
            direction_history: vec![],
        }
    }

    fn move_candidates(&self, grid: &Grid) -> Vec<Self> {
        Direction::DIRECTIONS
            .into_iter()
            .filter_map(|dir| {
                let mut candidate = self.clone();
                let is_valid = candidate.move_direction(grid, dir);
                if is_valid {
                    Some(candidate)
                } else {
                    None
                }
            })
            .collect()
    }

    fn move_direction(&mut self, grid: &Grid, direction: Direction) -> bool {
        if let Some(last_movement) = self.direction_history.last() {
            // Cancel if we just came from that direction
            if *last_movement == direction.reverse() {
                return false;
            }

            let straight_line_distance = self
                .direction_history
                .iter()
                .rev()
                .take_while(|dir| **dir == *last_movement)
                .count();

            // Cancel if we've been travelling in a straight line too long
            if *last_movement == direction && straight_line_distance >= 10 {
                return false;
            }

            // Cancel if we just turned and are trying to turn again
            if *last_movement != direction && straight_line_distance < 4 {
                return false;
            }
        }

        // Cancel if this movement takes us off the grid
        let Some(new_position) = grid.move_position(self.position, direction) else {
            return false;
        };

        self.direction_history.push(direction);
        self.direction_history
            .splice(0..self.direction_history.len().saturating_sub(10), []);
        self.position = new_position;

        true
    }

    fn heat_loss_at_position(&self, grid: &Grid) -> u32 {
        grid.get(self.position)
            .expect("crucible at invalid position")
    }

    fn is_finished(&self, grid: &Grid) -> bool {
        if let Some(last_direction) = self.direction_history.last() {
            let straight_line_distance = self
                .direction_history
                .iter()
                .rev()
                .take_while(|dir| *dir == last_direction)
                .count();

            // Can't stop unless we've been travelling in the same line for at
            // least 4 tiles
            if straight_line_distance < 4 {
                return false;
            }
        }

        self.position == grid.end()
    }
}
