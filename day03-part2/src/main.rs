use std::collections::HashMap;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = std::io::stdin().lock();
    let schematic = Schematic::new(input)?;

    let gear_ratios_sum: u32 = schematic.gear_ratios().sum();
    println!("{gear_ratios_sum}");

    Ok(())
}

struct Schematic {
    rows: Vec<Vec<u8>>,
}

impl Schematic {
    fn new(reader: impl std::io::BufRead) -> eyre::Result<Self> {
        let rows = reader
            .lines()
            .map(|line| {
                let line = line?;
                let row = line.into_bytes();
                Result::<_, eyre::Error>::Ok(row)
            })
            .collect::<eyre::Result<Vec<_>>>()?;

        Ok(Self { rows })
    }

    fn numbers(&self) -> impl Iterator<Item = SchematicNumber> + '_ {
        self.rows
            .iter()
            .enumerate()
            .flat_map(|(n, row)| SchematicNumberRowIter::new(row, n))
    }

    fn gear_ratios(&self) -> impl Iterator<Item = u32> + '_ {
        let mut gear_like_neighbor_numbers: HashMap<Position, Vec<SchematicNumber>> =
            HashMap::new();
        for number in self.numbers() {
            for (neighbor, neighbor_pos) in self.neighbors(number) {
                if neighbor == b'*' {
                    gear_like_neighbor_numbers
                        .entry(neighbor_pos)
                        .or_default()
                        .push(number);
                }
            }
        }

        gear_like_neighbor_numbers
            .into_iter()
            .filter_map(|(_, numbers)| {
                if numbers.len() == 2 {
                    Some(numbers[0].value * numbers[1].value)
                } else {
                    None
                }
            })
    }

    fn cell(&self, position: Position) -> Option<u8> {
        let row_index: usize = position.row.try_into().ok()?;
        let col_index: usize = position.col.try_into().ok()?;

        Some(*self.rows.get(row_index)?.get(col_index)?)
    }

    fn neighbors(&self, number: SchematicNumber) -> impl Iterator<Item = (u8, Position)> + '_ {
        let row_before = number.start.row - 1;
        let row_after = number.start.row + 1;
        let col_before = number.start.col - 1;
        let col_after = number.start.col + number.length;
        let above_positions = (col_before..=col_after).map(move |col| Position {
            row: row_before,
            col,
        });
        let below_positions = (col_before..=col_after).map(move |col| Position {
            row: row_after,
            col,
        });
        let next_to_positions = [
            Position {
                row: number.start.row,
                col: col_before,
            },
            Position {
                row: number.start.row,
                col: col_after,
            },
        ];

        let neighbor_positions = above_positions
            .chain(below_positions)
            .chain(next_to_positions);

        neighbor_positions.filter_map(|pos| Some((self.cell(pos)?, pos)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    row: isize,
    col: isize,
}

#[derive(Debug, Clone, Copy)]
struct SchematicNumber {
    value: u32,
    start: Position,
    length: isize,
}

struct SchematicNumberRowIter<'a> {
    row: &'a [u8],
    row_index: usize,
    index: usize,
}

impl<'a> SchematicNumberRowIter<'a> {
    fn new(row: &'a [u8], row_index: usize) -> Self {
        Self {
            row,
            row_index,
            index: 0,
        }
    }
}

impl<'a> Iterator for SchematicNumberRowIter<'a> {
    type Item = SchematicNumber;

    fn next(&mut self) -> Option<SchematicNumber> {
        let row: isize = self.row_index.try_into().expect("invalid row index");
        while self.index < self.row.len() {
            let remaining = &self.row[self.index..];
            if let Some((value, length)) = split_digit_prefix(&remaining) {
                let start = self.index;
                self.index += length;

                return Some(SchematicNumber {
                    value,
                    start: Position {
                        row: row,
                        col: start.try_into().expect("invlaid column index"),
                    },
                    length: length.try_into().expect("invalid length"),
                });
            }

            self.index += 1;
        }

        None
    }
}

fn split_digit_prefix(bytes: &[u8]) -> Option<(u32, usize)> {
    let prefix_length = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();

    if prefix_length >= 1 {
        let value = &bytes[0..prefix_length];
        let value = std::str::from_utf8(value).expect("invalid utf-8 sequence");
        let value = value.parse().expect("failed to parse number");

        Some((value, prefix_length))
    } else {
        None
    }
}
