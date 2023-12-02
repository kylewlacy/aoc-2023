use std::io::BufRead as _;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = std::io::stdin().lock();
    let power_sum = input
        .lines()
        .map(|line| -> eyre::Result<_> {
            let line = line?;
            let game: Game = line.parse()?;
            let power = game_power(&game);
            Ok(power)
        })
        .try_fold(0, |acc, power| Ok::<_, eyre::Error>(acc + power?))?;

    println!("{power_sum}");

    Ok(())
}

struct Game {
    #[allow(unused)]
    id: u32,
    sets: Vec<CubeSet>,
}

impl std::str::FromStr for Game {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("Game ")
            .ok_or_else(|| eyre::eyre!("invalid game string"))?;
        let (id, s) = s
            .split_once(": ")
            .ok_or_else(|| eyre::eyre!("invlaid game string"))?;
        let id = id.parse()?;

        let sets = s
            .split("; ")
            .map(|s| s.parse())
            .collect::<eyre::Result<Vec<_>>>()?;

        Ok(Self { id, sets })
    }
}

struct CubeSet {
    counts: Vec<CubeCount>,
}

impl std::str::FromStr for CubeSet {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let counts = s
            .split(", ")
            .map(|s| s.parse())
            .collect::<eyre::Result<Vec<_>>>()?;

        Ok(Self { counts })
    }
}

enum CubeCount {
    Red(u32),
    Green(u32),
    Blue(u32),
}

impl std::str::FromStr for CubeCount {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (count, color) = s
            .split_once(' ')
            .ok_or_else(|| eyre::eyre!("invalid cube count"))?;
        let count = count.parse()?;
        match color {
            "red" => Ok(Self::Red(count)),
            "green" => Ok(Self::Green(count)),
            "blue" => Ok(Self::Blue(count)),
            other => {
                eyre::bail!("invalid cube color: {other:?}");
            }
        }
    }
}

#[derive(Debug, Default)]
struct CubeCounts {
    reds: u32,
    greens: u32,
    blues: u32,
}

impl CubeCounts {
    fn observe(&mut self, count: &CubeCount) {
        match count {
            CubeCount::Red(reds) => self.reds = std::cmp::max(*reds, self.reds),
            CubeCount::Green(greens) => self.greens = std::cmp::max(*greens, self.greens),
            CubeCount::Blue(blues) => self.blues = std::cmp::max(*blues, self.blues),
        }
    }

    fn power(&self) -> u32 {
        self.reds * self.greens * self.blues
    }
}

fn game_power(game: &Game) -> u32 {
    let mut counts = CubeCounts::default();

    for set in &game.sets {
        for count in &set.counts {
            counts.observe(count);
        }
    }

    counts.power()
}
