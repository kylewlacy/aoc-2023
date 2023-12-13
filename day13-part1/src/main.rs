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

    let maps = parse_all(&input)?;
    let mut summary = 0;
    for map in maps {
        for col in 0..map.num_cols() {
            if map.is_vertical_reflection(col) {
                summary += col + 1;
            }
        }

        for row in 0..map.num_rows() {
            if map.is_horizontal_reflection(row) {
                summary += (row + 1) * 100;
            }
        }
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

#[derive(Debug)]
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
