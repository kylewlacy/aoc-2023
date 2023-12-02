use std::io::BufRead as _;

const MAX_COUNTS: CubeCounts = CubeCounts {
    reds: 12,
    greens: 13,
    blues: 14,
};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let input = std::io::stdin().lock();
    let games = input
        .lines()
        .map(|line| -> eyre::Result<_> {
            let line = line?;
            let game: Game = line.parse()?;
            Ok(game)
        })
        .collect::<eyre::Result<Vec<_>>>()?;
    let possible_games = games
        .into_iter()
        .filter(|game| is_game_possible(game, &MAX_COUNTS));
    let id_sum: u32 = possible_games.map(|game| game.id).sum();

    println!("{id_sum}");

    Ok(())
}

struct Game {
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

struct CubeCounts {
    reds: u32,
    greens: u32,
    blues: u32,
}

fn is_game_possible(game: &Game, max_counts: &CubeCounts) -> bool {
    game.sets.iter().all(|set| is_set_possible(set, max_counts))
}

fn is_set_possible(set: &CubeSet, max_counts: &CubeCounts) -> bool {
    set.counts.iter().all(|count| match &count {
        CubeCount::Red(red) => *red <= max_counts.reds,
        CubeCount::Green(green) => *green <= max_counts.greens,
        CubeCount::Blue(blue) => *blue <= max_counts.blues,
    })
}
