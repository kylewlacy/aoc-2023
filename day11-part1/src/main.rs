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

    let mut image: GalaxyImage = input.parse()?;

    println!("image:\n{image}");

    image.expand();

    println!("expanded:\n{image}");

    Ok(())
}

struct GalaxyImage {
    rows: Vec<Vec<Pixel>>,
}

impl GalaxyImage {
    fn expand(&mut self) {
        let num_rows = self.rows.len();
        let num_cols = self.rows.first().map(|row| row.len()).unwrap_or(0);

        for i in (0..num_rows).rev() {
            if self.rows[i].iter().all(|cell| *cell == Pixel::Empty) {
                self.rows.insert(i, vec![Pixel::Empty; num_cols]);
            }
        }

        for j in (0..num_cols).rev() {
            if self.rows.iter().all(|row| row[j] == Pixel::Empty) {
                for row in &mut self.rows {
                    row.insert(j, Pixel::Empty);
                }
            }
        }
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
