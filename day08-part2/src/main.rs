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

    let mut lines = input.lines();
    let directions = lines.next().ok_or_else(|| eyre::eyre!("invalid input"))?;
    let directions = directions
        .chars()
        .map(Direction::try_from)
        .collect::<eyre::Result<Vec<_>>>()?;

    let _ = lines.next();

    let mut nodes = HashMap::new();

    for line in lines {
        let (node, line) = line
            .split_once(" = (")
            .ok_or_else(|| eyre::eyre!("invalid node"))?;
        let (left, line) = line
            .split_once(", ")
            .ok_or_else(|| eyre::eyre!("invalid node"))?;
        let right = line
            .strip_suffix(")")
            .ok_or_else(|| eyre::eyre!("invalid node"))?;

        nodes.insert(node, (left, right));
    }

    let ghost_positions = nodes.keys().copied().filter(|node| node.ends_with('A'));
    let ghost_steps: Vec<_> = ghost_positions
        .map(|position| steps_to_exit(&nodes, &directions, position))
        .collect();

    let ghost_steps_lcm =
        lcmx::lcmx(&ghost_steps).ok_or_else(|| eyre::eyre!("failed to get lcm"))?;

    println!("{ghost_steps_lcm}");

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = eyre::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            value => {
                eyre::bail!("invalid direction: {value:?}");
            }
        }
    }
}

fn steps_to_exit(
    nodes: &HashMap<&str, (&str, &str)>,
    directions: &[Direction],
    start: &str,
) -> u64 {
    let mut position = start;
    let mut steps = 0;
    for direction in std::iter::repeat(directions).flatten() {
        if position.ends_with('Z') {
            return steps;
        }

        let node = nodes
            .get(position)
            .ok_or_else(|| eyre::eyre!("node not found: {position:?}"))
            .unwrap();

        match direction {
            Direction::Left => position = node.0,
            Direction::Right => position = node.1,
        }

        steps += 1;
    }

    unreachable!("unexpected end of infinite iterator");
}
