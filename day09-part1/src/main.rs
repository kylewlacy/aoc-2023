#![feature(array_windows)]

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

    let next_in_sequence_sum = input
        .lines()
        .map(|sequence| {
            let sequence = sequence
                .split_whitespace()
                .map(|value| Ok(value.parse()?))
                .collect::<eyre::Result<Vec<i32>>>()?;
            let next = next_in_sequence(&sequence);

            eyre::Ok(next)
        })
        .try_fold(0, |acc, value| eyre::Ok(acc + value?))?;

    println!("{next_in_sequence_sum}");

    Ok(())
}

fn next_in_sequence(sequence: &[i32]) -> i32 {
    let diffs = sequence.array_windows().map(|[a, b]| b - a);
    let first_diff = diffs.clone().next().unwrap_or(0);
    let last = sequence.last().copied().unwrap_or(0);
    if diffs.clone().all(|diff| diff == first_diff) {
        last + first_diff
    } else {
        let diffs: Vec<_> = diffs.collect();
        let next_diff = next_in_sequence(&diffs);
        last + next_diff
    }
}
