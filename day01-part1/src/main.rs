use std::io::BufRead as _;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = std::io::stdin().lock();

    let mut values = input.lines().map(|line| -> eyre::Result<_> {
        let line = line?;
        let mut bytes = line.bytes();
        let first_digit = bytes.find_map(to_digit);
        let first_digit = first_digit.ok_or_else(|| eyre::eyre!("digit not found"))?;
        let last_digit = bytes.rev().find_map(to_digit).unwrap_or(first_digit);

        let value = (first_digit * 10) + last_digit;

        Ok(value)
    });
    let sum = values.try_fold(0, |acc, value| Ok::<_, eyre::Error>(acc + value?))?;

    println!("{sum}");

    Ok(())
}

fn to_digit(byte: u8) -> Option<u32> {
    char::from(byte).to_digit(10)
}
