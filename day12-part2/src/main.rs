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

    let mut rows = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<Vec<Row>>>()?;

    for row in &mut rows {
        row.unfold();
    }

    tracing::info!("starting");
    let total_solutions: usize = rows
        .iter()
        .enumerate()
        .map(|(n, row)| {
            let solutions = num_solutions(&row.cells, &row.constraints, Contiguity::Normal);
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
    constraints: Vec<u32>,
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Contiguity {
    #[default]
    Normal,
    ContinuesGroup,
    BreaksGroup,
}

fn num_solutions(cells: &[PartialCell], constraints: &[u32], contiguity: Contiguity) -> usize {
    let num_solutions = compute_num_solutions(cells, constraints, contiguity);

    num_solutions
}

fn compute_num_solutions(
    cells: &[PartialCell],
    constraints: &[u32],
    contiguity: Contiguity,
) -> usize {
    if constraints.is_empty() {
        if cells.iter().all(|cell| *cell != PartialCell::Damaged) {
            return 1;
        } else {
            return 0;
        }
    }

    match cells {
        &[] => 0,
        &[PartialCell::Operational, ref rest @ ..] => match contiguity {
            Contiguity::ContinuesGroup => 0,
            Contiguity::Normal | Contiguity::BreaksGroup => {
                num_solutions(rest, constraints, Contiguity::Normal)
            }
        },
        &[PartialCell::Damaged, ..] => {
            match contiguity {
                Contiguity::Normal | Contiguity::ContinuesGroup => {}
                Contiguity::BreaksGroup => {
                    return 0;
                }
            }

            let Some((constraint, rest_constraints)) = constraints.split_first() else {
                return 0;
            };
            let damaged_split_index = cells
                .iter()
                .enumerate()
                .find_map(|(n, cell)| {
                    if *cell != PartialCell::Damaged {
                        Some(n)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| cells.len());
            let (damaged, rest) = cells.split_at(damaged_split_index);
            let num_damaged: u32 = damaged.len().try_into().unwrap();

            match num_damaged.cmp(constraint) {
                std::cmp::Ordering::Less => {
                    let contiguous_constraints = [constraint - num_damaged]
                        .into_iter()
                        .chain(rest_constraints.iter().copied())
                        .collect::<Vec<_>>();
                    num_solutions(rest, &contiguous_constraints, Contiguity::ContinuesGroup)
                }
                std::cmp::Ordering::Equal => {
                    num_solutions(rest, rest_constraints, Contiguity::BreaksGroup)
                }
                std::cmp::Ordering::Greater => 0,
            }
        }
        &[PartialCell::Unknown, ref rest @ ..] => {
            let a = vec![PartialCell::Damaged]
                .into_iter()
                .chain(rest.iter().copied())
                .collect::<Vec<_>>();
            let a_solutions = num_solutions(&a, constraints, contiguity);
            let b = [PartialCell::Operational]
                .into_iter()
                .chain(rest.iter().copied())
                .collect::<Vec<_>>();
            let b_solutions = num_solutions(&b, constraints, contiguity);
            a_solutions + b_solutions
        }
    }
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
            Self::Operational => write!(f, "."),
            Self::Damaged => write!(f, "#"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

struct DisplayCells<'a>(&'a [PartialCell]);

impl std::fmt::Display for DisplayCells<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for cell in self.0 {
            match cell {
                PartialCell::Operational => string.push('.'),
                PartialCell::Damaged => string.push('#'),
                PartialCell::Unknown => string.push('?'),
            }
        }

        f.pad(&string)?;

        Ok(())
    }
}
