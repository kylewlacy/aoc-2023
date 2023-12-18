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

    let instructions: Vec<Instruction> = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<_>>()?;

    let mut position = Position { x: 0.0, y: 0.0 };
    let mut points = vec![position];
    let mut perimieter = 0.0;
    for instruction in &instructions {
        let new_pos = position.offset(instruction.direction, instruction.distance);
        perimieter += instruction.distance;

        points.push(new_pos);
        position = new_pos;
    }

    let inner_volume = polygon_area(&points);
    let full_volume = inner_volume + (perimieter / 2.0) + 1.0;
    println!("{full_volume}");

    Ok(())
}

fn polygon_area(points: &[Position]) -> f64 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area = area + points[i].x as f64 * points[j].y as f64;
        area = area - points[i].y as f64 * points[j].x as f64;
    }

    area / 2.0
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f64,
    y: f64,
}

impl Position {
    fn offset(&self, direction: Direction, distance: f64) -> Position {
        match direction {
            Direction::Up => Position {
                x: self.x,
                y: self.y - distance,
            },
            Direction::Down => Position {
                y: self.y + distance,
                x: self.x,
            },
            Direction::Left => Position {
                y: self.y,
                x: self.x - distance,
            },
            Direction::Right => Position {
                y: self.y,
                x: self.x + distance,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    direction: Direction,
    distance: f64,
}

impl std::str::FromStr for Instruction {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_string = s
            .split_whitespace()
            .nth(2)
            .ok_or_eyre("invalid instruction")?;
        let hex_string = hex_string
            .strip_prefix('(')
            .ok_or_eyre("invalid instruction")?
            .strip_suffix(')')
            .ok_or_eyre("invalid instruction")?;
        let hex_digits = hex_string
            .strip_prefix('#')
            .ok_or_eyre("invalid instruction")?;

        let (distance_hex, direction_hex) = hex_digits.split_at(5);
        let direction = match direction_hex {
            "0" => Direction::Right,
            "1" => Direction::Down,
            "2" => Direction::Left,
            "3" => Direction::Up,
            other => {
                eyre::bail!("invalid direction hex: {other:?}");
            }
        };
        let distance = u64::from_str_radix(distance_hex, 16)?;
        let distance = distance as f64;

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
