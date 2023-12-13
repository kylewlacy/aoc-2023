use std::{collections::HashSet, io::Read as _};

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

    let maps = parse_all(&input)?;
    let mut summary = 0;
    for map in maps {
        let unsmudged_reflections = map.unsmudged_reflections()?;
        summary += unsmudged_reflections
            .iter()
            .map(|reflection| reflection.value())
            .sum::<usize>();
    }

    println!("{summary}");

    Ok(())
}

fn parse_all(s: &str) -> eyre::Result<Vec<Map>> {
    let maps = s
        .split("\n\n")
        .map(|map| map.parse())
        .collect::<eyre::Result<Vec<_>>>()?;
    Ok(maps)
}

#[derive(Debug, Clone)]
struct Map {
    rows: Vec<Vec<Cell>>,
}

impl Map {
    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn is_vertical_reflection(&self, col: usize) -> bool {
        let mut result = false;

        let mut i = 0;
        loop {
            let Some(left) = col.checked_sub(i) else {
                break;
            };
            let right = col + i + 1;
            if right >= self.num_cols() {
                break;
            }

            for row in &self.rows {
                if row[left] != row[right] {
                    return false;
                }
            }

            i += 1;
            result = true;
        }

        result
    }

    fn is_horizontal_reflection(&self, row: usize) -> bool {
        let mut result = false;

        let mut i = 0;
        loop {
            let Some(left) = row.checked_sub(i) else {
                break;
            };
            let right = row + i + 1;
            if right >= self.num_rows() {
                break;
            }

            let left_row = &self.rows[left];
            let right_row = &self.rows[right];
            for (a, b) in left_row.iter().zip(right_row.iter()) {
                if a != b {
                    return false;
                }
            }

            i += 1;
            result = true;
        }

        result
    }

    fn unsmudged_reflections(&self) -> eyre::Result<Vec<Reflection>> {
        let reflections = self.reflections().collect::<HashSet<_>>();
        for row in 0..self.num_rows() {
            for col in 0..self.num_cols() {
                let mut candidate = self.clone();
                candidate.rows[row][col].flip();

                let new_candidate_reflections = candidate
                    .reflections()
                    .filter(|reflection| !reflections.contains(reflection))
                    .collect::<Vec<_>>();

                if !new_candidate_reflections.is_empty() {
                    return Ok(new_candidate_reflections);
                }
            }
        }

        eyre::bail!("smudge not found for map");
    }

    fn reflections(&self) -> impl Iterator<Item = Reflection> + '_ {
        let vertical_reflections = (0..self.num_cols()).filter_map(|col| {
            if self.is_vertical_reflection(col) {
                Some(Reflection::Vertical { col })
            } else {
                None
            }
        });
        let horizontal_reflections = (0..self.num_rows()).filter_map(|row| {
            if self.is_horizontal_reflection(row) {
                Some(Reflection::Horizontal { row })
            } else {
                None
            }
        });

        vertical_reflections.chain(horizontal_reflections)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Reflection {
    Vertical { col: usize },
    Horizontal { row: usize },
}

impl Reflection {
    fn value(&self) -> usize {
        match self {
            Reflection::Vertical { col } => col + 1,
            Reflection::Horizontal { row } => (row + 1) * 100,
        }
    }
}

impl std::str::FromStr for Map {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(Cell::try_from)
                    .collect::<eyre::Result<Vec<_>>>()
            })
            .collect::<eyre::Result<Vec<Vec<_>>>>()?;

        Ok(Self { rows })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Ash,
    Rock,
}

impl Cell {
    fn flip(&mut self) {
        match self {
            Self::Ash => {
                *self = Self::Rock;
            }
            Self::Rock => {
                *self = Self::Ash;
            }
        }
    }
}

impl TryFrom<char> for Cell {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Ash),
            '#' => Ok(Self::Rock),
            other => {
                eyre::bail!("invalid cell: {other:?}");
            }
        }
    }
}
