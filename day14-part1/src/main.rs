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

    let mut platform: Platform = input.parse()?;
    println!("=== Start ===");
    println!("{platform}");

    let mut moved;
    loop {
        moved = platform.roll_up();

        if !moved {
            break;
        }

        println!();
        println!("=== Moved ===");
        println!("{platform}");
    }

    println!();
    println!("=== Finished ===");
    println!("{platform}");

    let total_load = platform.total_load();
    println!("{total_load}");

    Ok(())
}

struct Platform {
    rows: Vec<Vec<Cell>>,
}

impl Platform {
    fn roll_up(&mut self) -> bool {
        let mut moved = false;
        for i in 1..self.rows.len() {
            for j in 0..self.rows[i].len() {
                match (self.rows[i - 1][j], self.rows[i][j]) {
                    (Cell::Space, Cell::Rock) => {
                        self.rows[i - 1][j] = Cell::Rock;
                        self.rows[i][j] = Cell::Space;
                        moved = true;
                    }
                    _ => {}
                }
            }
        }

        moved
    }

    fn total_load(&self) -> usize {
        let num_rows = self.rows.len();
        self.rows
            .iter()
            .enumerate()
            .map(|(n, row)| {
                let num_rocks = row.iter().filter(|cell| **cell == Cell::Rock).count();
                num_rocks * (num_rows - n)
            })
            .sum()
    }
}

impl std::str::FromStr for Platform {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rows = s
            .lines()
            .map(|line| {
                line.chars()
                    .map(Cell::try_from)
                    .collect::<eyre::Result<Vec<_>>>()
            })
            .collect::<eyre::Result<_>>()?;
        Ok(Self { rows })
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, row) in self.rows.iter().enumerate() {
            if n != 0 {
                writeln!(f)?;
            }

            for cell in row {
                write!(f, "{cell}")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Space,
    CubeRock,
    Rock,
}

impl TryFrom<char> for Cell {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Space),
            '#' => Ok(Self::CubeRock),
            'O' => Ok(Self::Rock),
            other => {
                eyre::bail!("invalid character: {other:?}");
            }
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Space => '.',
            Cell::CubeRock => '#',
            Cell::Rock => 'O',
        };

        write!(f, "{c}")
    }
}
