use std::io::Read;

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

    let race = Race::parse(&input)?;
    let ways_to_win = race.ways_to_win();

    println!("{ways_to_win}");

    Ok(())
}

#[derive(Debug)]
struct Race {
    time: u64,
    distance_record: u64,
}

impl Race {
    fn parse(input: &str) -> eyre::Result<Self> {
        let mut lines = input.lines();
        let times = lines
            .next()
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let time = times
            .strip_prefix("Time:")
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let time = time
            .split_whitespace()
            .flat_map(|s| s.chars())
            .collect::<String>()
            .parse()?;

        let distance_record = lines
            .next()
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let distance_record = distance_record
            .strip_prefix("Distance:")
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let distance_record = distance_record
            .split_whitespace()
            .flat_map(|s| s.chars())
            .collect::<String>()
            .parse()?;

        Ok(Race {
            time,
            distance_record,
        })
    }

    fn ways_to_win(&self) -> u64 {
        let mut ways_to_win = 0;
        for hold_duration in 0..self.time {
            let move_duration = self.time - hold_duration;
            let distance_moved = move_duration * hold_duration;
            if distance_moved > self.distance_record {
                ways_to_win += 1
            }
        }

        ways_to_win
    }
}
