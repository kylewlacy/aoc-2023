use std::io::BufRead as _;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = std::io::stdin().lock();

    let mut values = input.lines().map(|line| -> eyre::Result<_> {
        let line = line?;
        let mut digits = find_digits(&line);
        let first_digit = digits
            .next()
            .ok_or_else(|| eyre::eyre!("digit not found"))?;
        let last_digit = digits.last().unwrap_or(first_digit);

        let value = (first_digit * 10) + last_digit;

        Ok(value)
    });
    let sum = values.try_fold(0, |acc, value| Ok::<_, eyre::Error>(acc + value?))?;

    println!("{sum}");

    Ok(())
}

fn find_digits(string: &str) -> impl Iterator<Item = u32> + '_ {
    FindDigitsIter {
        bytes: string.as_bytes(),
    }
}

struct FindDigitsIter<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for FindDigitsIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        while !self.bytes.is_empty() {
            let digit = digit_prefix(&self.bytes);

            // Unconditionally advance to the next character, so that we handle
            // strings like `oneight` (this should return both "1" and "8"
            // as digits).
            self.bytes = &self.bytes[1..];

            if let Some(digit) = digit {
                return Some(digit);
            }
        }

        None
    }
}

fn named_digit_prefix(bytes: &[u8]) -> Option<u32> {
    if bytes.starts_with(b"one") {
        Some(1)
    } else if bytes.starts_with(b"two") {
        Some(2)
    } else if bytes.starts_with(b"three") {
        Some(3)
    } else if bytes.starts_with(b"four") {
        Some(4)
    } else if bytes.starts_with(b"five") {
        Some(5)
    } else if bytes.starts_with(b"six") {
        Some(6)
    } else if bytes.starts_with(b"seven") {
        Some(7)
    } else if bytes.starts_with(b"eight") {
        Some(8)
    } else if bytes.starts_with(b"nine") {
        Some(9)
    } else {
        None
    }
}

fn digit_prefix(bytes: &[u8]) -> Option<u32> {
    if let Some(digit) = named_digit_prefix(bytes) {
        Some(digit)
    } else if let Some(&head) = bytes.first() {
        let digit = char::from(head).to_digit(10)?;
        Some(digit)
    } else {
        None
    }
}
