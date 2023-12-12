use std::{io::Read as _, str::FromStr};

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

    let rows = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<Vec<Row>>>()?;

    let total_solutions: usize = rows.iter().map(|row| row.num_solutions()).sum();
    println!("{total_solutions}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Row {
    cells: Vec<PartialCell>,
    constraints: Vec<u32>,
}

impl Row {
    fn state(&self) -> State {
        if self.cells.iter().any(|cell| *cell == PartialCell::Unknown) {
            State::Unsolved
        } else {
            let damaged_groups = self
                .cells
                .split(|cell| *cell == PartialCell::Operational)
                .filter(|group| !group.is_empty())
                .map(|group| -> u32 { group.len().try_into().unwrap() });
            if damaged_groups.eq(self.constraints.iter().copied()) {
                State::Solved
            } else {
                State::Invalid
            }
        }
    }

    fn num_solutions(&self) -> usize {
        let state = self.state();
        match state {
            State::Solved => {
                return 1;
            }
            State::Invalid => {
                return 0;
            }
            State::Unsolved => {}
        }

        let next_unknown_position = self
            .cells
            .iter()
            .enumerate()
            .find_map(|(n, cell)| match cell {
                PartialCell::Unknown => Some(n),
                PartialCell::Operational | PartialCell::Damaged => None,
            })
            .expect("no unknown positions for unsolved row");

        let mut a = self.clone();
        a.cells[next_unknown_position] = PartialCell::Operational;

        let mut b = self.clone();
        b.cells[next_unknown_position] = PartialCell::Damaged;

        a.num_solutions() + b.num_solutions()
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Solved,
    Unsolved,
    Invalid,
}

impl FromStr for Row {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cells, constraints) = s.split_once(' ').ok_or_eyre("invalid row")?;
        let cells = cells
            .chars()
            .map(PartialCell::try_from)
            .collect::<eyre::Result<_>>()?;
        let constraints = constraints
            .split(',')
            .map(|constraint| Ok(constraint.parse()?))
            .collect::<eyre::Result<_>>()?;

        Ok(Self { cells, constraints })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartialCell {
    Operational,
    Damaged,
    Unknown,
}

impl TryFrom<char> for PartialCell {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Damaged),
            '?' => Ok(Self::Unknown),
            other => {
                eyre::bail!("invalid cell: {other:?}");
            }
        }
    }
}
