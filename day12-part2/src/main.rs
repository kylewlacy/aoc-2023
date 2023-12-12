use std::{io::Read as _, str::FromStr};

use bitvec::array::BitArray;
use eyre::OptionExt;
use itertools::Itertools as _;
use rayon::iter::{
    IndexedParallelIterator as _, IntoParallelRefIterator as _, ParallelIterator as _,
};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();
    color_eyre::install()?;

    let mut stdin = std::io::stdin().lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input)?;

    let mut rows = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<Vec<Row>>>()?;

    for row in &mut rows {
        row.unfold();

        tracing::debug!(%row, cells = ?row.cells.len(), "unfolded");
    }

    tracing::info!("starting");
    let total_solutions: usize = rows
        .par_iter()
        .enumerate()
        .map(|(n, row)| {
            let solutions = row.num_solutions();
            tracing::info!("row {n}: {solutions} solution(s)");
            solutions
        })
        .sum();
    tracing::info!("complete");
    println!("{total_solutions}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Row {
    cells: Vec<PartialCell>,
    constraints: Vec<u8>,
}

impl Row {
    fn unfold(&mut self) {
        let new_cells = (0..5)
            .flat_map(|_| {
                [PartialCell::Unknown]
                    .into_iter()
                    .chain(self.cells.iter().copied())
            })
            .skip(1)
            .collect();
        let new_constraints = (0..5)
            .flat_map(|_| self.constraints.iter().cloned())
            .collect();

        self.cells = new_cells;
        self.constraints = new_constraints;
    }

    fn state(&self) -> State {
        if self.cells.iter().any(|cell| *cell == PartialCell::Unknown) {
            State::Unsolved
        } else {
            let damaged_groups = self
                .cells
                .split(|cell| *cell == PartialCell::Operational)
                .filter(|group| !group.is_empty())
                .map(|group| -> u8 { group.len().try_into().unwrap() });
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

        let unknown_positions =
            self.cells
                .iter()
                .enumerate()
                .filter_map(|(n, cell)| -> Option<u8> {
                    match cell {
                        PartialCell::Unknown => Some(n.try_into().unwrap()),
                        PartialCell::Operational | PartialCell::Damaged => None,
                    }
                });

        let mut initial_candidate_row = CompleteRow::new();
        for (n, cell) in self.cells.iter().enumerate() {
            let initial_candidiate = match cell {
                PartialCell::Operational | PartialCell::Unknown => CompleteCell::Operational,
                PartialCell::Damaged => CompleteCell::Damaged,
            };
            initial_candidate_row.set(n.try_into().unwrap(), initial_candidiate);
        }

        unknown_positions
            .powerset()
            .filter(|flip_positions| {
                let mut candidate_row = initial_candidate_row;

                for position in flip_positions {
                    candidate_row.set(*position, CompleteCell::Damaged);
                }
                candidate_row.matches_constraints(&self.constraints)
            })
            .count()
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cell in &self.cells {
            write!(f, "{cell}")?;
        }

        if !self.constraints.is_empty() && !self.cells.is_empty() {
            write!(f, " ")?;
        }

        for (n, constraint) in self.constraints.iter().enumerate() {
            if n != 0 {
                write!(f, ",")?;
            }

            write!(f, "{constraint}")?;
        }

        Ok(())
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

impl std::fmt::Display for PartialCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartialCell::Operational => write!(f, "."),
            PartialCell::Damaged => write!(f, "#"),
            PartialCell::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CompleteRow(BitArray<[u64; 2]>);

impl CompleteRow {
    fn new() -> Self {
        Self(BitArray::new([0; 2]))
    }

    fn matches_constraints(&self, constraints: &[u8]) -> bool {
        let groups = self
            .0
            .split(|_, bit| !bit)
            .filter(|group| !group.is_empty())
            .map(|group| group.len() as u8);
        groups.eq(constraints.iter().copied())
    }

    fn set(&mut self, index: u8, value: CompleteCell) {
        let bit = match value {
            CompleteCell::Operational => false,
            CompleteCell::Damaged => true,
        };
        self.0.set(index as usize, bit);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum CompleteCell {
    Operational = 0,
    Damaged = 1,
}
