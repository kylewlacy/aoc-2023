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

    let races = Race::parse_list(&input)?;
    let total_ways_to_win: u64 = races.iter().map(|race| race.ways_to_win()).product();

    println!("{total_ways_to_win}");

    Ok(())
}

#[derive(Debug)]
struct Race {
    time: u64,
    distance_record: u64,
}

impl Race {
    fn parse_list(input: &str) -> eyre::Result<Vec<Self>> {
        let mut lines = input.lines();
        let times = lines
            .next()
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let times = times
            .strip_prefix("Time:")
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let times = times.split_whitespace().map(|time| time.parse());

        let distance_records = lines
            .next()
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let distance_records = distance_records
            .strip_prefix("Distance:")
            .ok_or_else(|| eyre::eyre!("invalid race list"))?;
        let distance_records = distance_records
            .split_whitespace()
            .map(|distance| distance.parse());

        times
            .zip(distance_records)
            .map(|(time, distance)| {
                Ok(Race {
                    time: time?,
                    distance_record: distance?,
                })
            })
            .collect()
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
