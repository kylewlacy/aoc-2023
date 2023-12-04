use std::{
    collections::{HashSet, VecDeque},
    io::BufRead as _,
};

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
    let mut collected_card_ids = vec![];
    let mut pending_card_ids: VecDeque<usize> = (0..cards.len()).collect();

    while let Some(card_id) = pending_card_ids.pop_front() {
        collected_card_ids.push(card_id);
        let num_matches = cards[card_id].num_matches;
        let won_cards = (0..=num_matches).skip(1).map(|offset| card_id + offset);
        pending_card_ids.extend(won_cards);
    }

    println!("{}", collected_card_ids.len());

    Ok(())
}

struct Card {
    num_matches: usize,
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

        let num_matches = ours.iter().filter(|num| winners.contains(num)).count();

        Ok(Self { num_matches })
    }
}
