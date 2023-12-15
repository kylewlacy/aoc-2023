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

    let hash_sum: u64 = input
        .lines()
        .next()
        .unwrap_or("")
        .split(",")
        .map(hash)
        .sum();

    println!("{hash_sum}");

    Ok(())
}

fn hash(s: &str) -> u64 {
    let mut value = 0;
    for c in s.chars() {
        let ascii: u8 = c.try_into().expect("invalid ASCII");
        let ascii: u64 = ascii.into();
        value += ascii;
        value *= 17;
        value %= 256;
    }

    value
}
