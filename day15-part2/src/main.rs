use std::io::Read as _;

use eyre::OptionExt;

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

    let steps = input.lines().next().ok_or_eyre("no steps")?;
    let steps = steps.split(",").map(Step::parse);

    let mut boxes = vec![LensBox::default(); 256];

    for step in steps {
        let step = step?;
        let hash = hash(step.label());
        let hash: usize = hash.try_into().expect("invalid hash");
        boxes[hash].apply(&step);
    }

    let total_focusing_power: u64 = boxes
        .iter()
        .enumerate()
        .flat_map(|(i, box_)| {
            box_.lenses
                .iter()
                .enumerate()
                .map(move |(j, (_, value))| -> u64 {
                    let i: u64 = i.try_into().unwrap();
                    let j: u64 = j.try_into().unwrap();
                    let value: u64 = (*value).into();
                    (i + 1) * (j + 1) * value
                })
        })
        .sum();

    println!("{total_focusing_power}");

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

#[derive(Debug)]
enum Step<'a> {
    Add { label: &'a str, value: u8 },
    Remove { label: &'a str },
}

impl<'a> Step<'a> {
    fn parse(s: &'a str) -> eyre::Result<Self> {
        if let Some((label, value)) = s.split_once('=') {
            let value = value.parse()?;
            Ok(Self::Add { label, value })
        } else if let Some(label) = s.strip_suffix('-') {
            Ok(Self::Remove { label })
        } else {
            eyre::bail!("invalid step: {s:?}");
        }
    }

    fn label(&self) -> &str {
        match self {
            Step::Add { label, value: _ } => label,
            Step::Remove { label } => label,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LensBox<'a> {
    lenses: Vec<(&'a str, u8)>,
}

impl<'a> LensBox<'a> {
    fn apply(&mut self, step: &Step<'a>) {
        match step {
            Step::Add { label, value } => {
                if let Some(index) = self.lens_index(label) {
                    self.lenses[index] = (label, *value);
                } else {
                    self.lenses.push((label, *value));
                }
            }
            Step::Remove { label } => {
                if let Some(lens_index) = self.lens_index(label) {
                    self.lenses.remove(lens_index);
                }
            }
        }
    }

    fn lens_index(&self, label: &str) -> Option<usize> {
        self.lenses.iter().enumerate().find_map(
            |(i, (lens_label, _))| {
                if *lens_label == label {
                    Some(i)
                } else {
                    None
                }
            },
        )
    }
}
