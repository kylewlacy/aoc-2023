use std::{collections::HashSet, io::Read as _};

const EPXANSION_FACTOR: i64 = 999_999;

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

    let image: GalaxyImage = input.parse()?;
    let galaxies = image.galaxies();

    let galaxy_pairs = galaxies
        .iter()
        .flat_map(|a| {
            galaxies.iter().filter_map(move |b| match a.cmp(&b) {
                std::cmp::Ordering::Less => Some((a, b)),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some((b, a)),
            })
        })
        .collect::<HashSet<_>>();

    let sum: i64 = galaxy_pairs.iter().map(|(a, b)| a.distance_to(b)).sum();

    println!("{sum}");

    Ok(())
}

struct GalaxyImage {
    rows: Vec<Vec<Pixel>>,
}

impl GalaxyImage {
    fn galaxies(&self) -> Vec<Position> {
        let num_rows = self.rows.len();
        let num_cols = self.rows.first().map(|row| row.len()).unwrap_or(0);

        let mut row_expansions = vec![];
        let mut col_expansions = vec![];

        for i in 0..num_rows {
            if self.rows[i].iter().all(|cell| *cell == Pixel::Empty) {
                row_expansions.push(i);
            }
        }

        for j in 0..num_cols {
            if self.rows.iter().all(|row| row[j] == Pixel::Empty) {
                col_expansions.push(j);
            }
        }

        let row_expansions = &row_expansions;
        let col_expansions = &col_expansions;

        self.rows
            .iter()
            .enumerate()
            .flat_map(|(row, cells)| {
                cells.iter().enumerate().filter_map(move |(col, cell)| {
                    let num_row_expansions =
                        row_expansions.iter().take_while(|i| **i < row).count();
                    let num_col_expansions =
                        col_expansions.iter().take_while(|j| **j < col).count();

                    match cell {
                        Pixel::Empty => None,
                        Pixel::Galaxy => Some(Position {
                            row: (row as i64) + (num_row_expansions as i64 * EPXANSION_FACTOR),
                            col: (col as i64) + (num_col_expansions as i64 * EPXANSION_FACTOR),
                        }),
                    }
                })
            })
            .collect()
    }
}

impl std::str::FromStr for GalaxyImage {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(Pixel::try_from)
                    .collect::<eyre::Result<Vec<_>>>()
            })
            .collect::<eyre::Result<Vec<Vec<_>>>>()?;

        Ok(Self { rows })
    }
}

impl std::fmt::Display for GalaxyImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            for cell in row {
                write!(f, "{cell}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pixel {
    Empty,
    Galaxy,
}

impl TryFrom<char> for Pixel {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            '#' => Ok(Self::Galaxy),
            other => {
                eyre::bail!("invalid pixel: {other:?}");
            }
        }
    }
}

impl std::fmt::Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Galaxy => write!(f, "#"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Position {
    row: i64,
    col: i64,
}

impl Position {
    fn distance_to(&self, other: &Self) -> i64 {
        let row_diff = self.row - other.row;
        let col_diff = self.col - other.col;

        i64::abs(row_diff) + i64::abs(col_diff)
    }
}
