use std::{collections::HashMap, io::Read as _};

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

    let mut bids = input
        .lines()
        .map(|line| {
            let mut parts = line.split_whitespace();
            let hand = parts.next().ok_or_else(|| eyre::eyre!("invalid line"))?;
            let hand: Hand = hand.parse()?;

            let amount = parts.next().ok_or_else(|| eyre::eyre!("invalid line"))?;
            let amount = amount.parse()?;

            Ok(Bid { hand, amount })
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    bids.sort_by(|a, b| a.hand.cmp(&b.hand));

    let winnings = bids.iter().enumerate().map(|(i, bid)| {
        let i: u32 = i.try_into().unwrap();
        let rank = i + 1;
        rank * bid.amount
    });
    let total_winnings: u32 = winnings.sum();

    println!("{total_winnings}");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Card {
    Joker,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Queen,
    King,
    Ace,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Card::Joker => write!(f, "J"),
            Card::Two => write!(f, "2"),
            Card::Three => write!(f, "3"),
            Card::Four => write!(f, "4"),
            Card::Five => write!(f, "5"),
            Card::Six => write!(f, "6"),
            Card::Seven => write!(f, "7"),
            Card::Eight => write!(f, "8"),
            Card::Nine => write!(f, "9"),
            Card::Ten => write!(f, "T"),
            Card::Queen => write!(f, "Q"),
            Card::King => write!(f, "K"),
            Card::Ace => write!(f, "A"),
        }
    }
}

impl TryFrom<char> for Card {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::Ace),
            'K' => Ok(Self::King),
            'Q' => Ok(Self::Queen),
            'T' => Ok(Self::Ten),
            '9' => Ok(Self::Nine),
            '8' => Ok(Self::Eight),
            '7' => Ok(Self::Seven),
            '6' => Ok(Self::Six),
            '5' => Ok(Self::Five),
            '4' => Ok(Self::Four),
            '3' => Ok(Self::Three),
            '2' => Ok(Self::Two),
            'J' => Ok(Self::Joker),
            other => eyre::bail!("invalid card: {other}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Cards([Card; 5]);

impl std::fmt::Display for Cards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c, d, e] = &self.0;
        write!(f, "{a}{b}{c}{d}{e}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    hand_type: HandType,
    cards: Cards,
}

impl Hand {
    fn new(cards: Cards) -> Self {
        let hand_type = hand_type(cards);
        Self { hand_type, cards }
    }
}

impl std::str::FromStr for Hand {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let cards = [
            Card::try_from(chars.next().ok_or_else(|| eyre::eyre!("invalid hand"))?)?,
            Card::try_from(chars.next().ok_or_else(|| eyre::eyre!("invalid hand"))?)?,
            Card::try_from(chars.next().ok_or_else(|| eyre::eyre!("invalid hand"))?)?,
            Card::try_from(chars.next().ok_or_else(|| eyre::eyre!("invalid hand"))?)?,
            Card::try_from(chars.next().ok_or_else(|| eyre::eyre!("invalid hand"))?)?,
        ];
        eyre::ensure!(chars.next().is_none(), "too many cards in hand");

        Ok(Self::new(Cards(cards)))
    }
}

#[derive(Debug, Clone, Copy)]
struct Bid {
    hand: Hand,
    amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

fn hand_type(cards: Cards) -> HandType {
    let mut counts: HashMap<Card, u8> = HashMap::new();
    let mut jokers = 0;
    for card in &cards.0 {
        match card {
            Card::Joker => {
                jokers += 1;
            }
            card => {
                let count = counts.entry(*card).or_default();
                *count += 1;
            }
        }
    }

    let mut counts: Vec<_> = counts.values().copied().collect();
    counts.sort_by(|a, b| a.cmp(b).reverse());

    // Add the number of jokers to the highest card count (or insert the number
    // of jokers if there are no non-jokers). This "upgrades" the best hand
    // so far based on the number of jokers.
    if counts.is_empty() {
        counts.push(jokers);
    } else {
        counts[0] += jokers;
    }

    let hand_type = match &counts[..] {
        &[5, ..] => HandType::FiveOfAKind,
        &[4, ..] => HandType::FourOfAKind,
        &[3, 2, ..] => HandType::FullHouse,
        &[3, ..] => HandType::ThreeOfAKind,
        &[2, 2, ..] => HandType::TwoPair,
        &[2, ..] => HandType::OnePair,
        _ => HandType::HighCard,
    };

    tracing::debug!(%cards, ?hand_type, ?counts, "hand type");

    hand_type
}
