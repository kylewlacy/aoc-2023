use std::{collections::HashSet, io::BufRead as _};

fn main() -> eyre::Result<()> {
    let input = std::io::stdin().lock();
    let cards = input
        .lines()
        .map(|line| {
            let line = line?;
            let card: Card = line.parse()?;

            Ok(card)
        })
        .collect::<eyre::Result<Vec<_>>>()?;
    let total_points: u32 = cards.iter().map(|card| card.points()).sum();

    println!("{total_points}");

    Ok(())
}

struct Card {
    #[allow(unused)]
    winners: HashSet<u32>,
    ours: Vec<u32>,
}

impl std::str::FromStr for Card {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, s) = s
            .split_once(": ")
            .ok_or_else(|| eyre::eyre!("invalid card"))?;
        let (winners, ours) = s
            .split_once('|')
            .ok_or_else(|| eyre::eyre!("invalid card"))?;

        let winners = winners
            .split_whitespace()
            .map(|n| Ok(n.parse()?))
            .collect::<eyre::Result<HashSet<u32>>>()?;
        let ours = ours
            .split_whitespace()
            .map(|n| Ok(n.parse()?))
            .collect::<eyre::Result<Vec<u32>>>()?;

        Ok(Self { winners, ours })
    }
}

impl Card {
    fn points(&self) -> u32 {
        let mut points = None;

        for number in &self.ours {
            if self.winners.contains(number) {
                match &mut points {
                    Some(points) => *points *= 2,
                    None => points = Some(1),
                }
            }
        }

        points.unwrap_or(0)
    }
}
