use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Read as _,
};

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

    let bricks = input
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<Vec<Brick>>>()?;
    let mut space = Space::new(bricks);
    space.settle();

    // Uncomment to print final brick positions:
    // for brick in &space.bricks {
    //     println!(
    //         "{},{},{}~{},{},{}",
    //         brick.start.x, brick.start.y, brick.start.z, brick.end.x, brick.end.y, brick.end.z
    //     );
    // }

    let disintegratable = space.num_disintegratable();
    println!("{disintegratable}");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Brick {
    start: Position,
    end: Position,
}

impl Brick {
    fn positions(&self) -> impl Iterator<Item = Position> {
        let axis;
        let min_axis;
        let max_axis;
        if self.start.x == self.end.x && self.start.y == self.end.y {
            axis = Axis::Z;
            min_axis = std::cmp::min(self.start.z, self.end.z);
            max_axis = std::cmp::max(self.start.z, self.end.z);
        } else if self.start.x == self.end.x && self.start.z == self.end.z {
            axis = Axis::Y;
            min_axis = std::cmp::min(self.start.y, self.end.y);
            max_axis = std::cmp::max(self.start.y, self.end.y);
        } else if self.start.y == self.end.y && self.start.z == self.end.z {
            axis = Axis::X;
            min_axis = std::cmp::min(self.start.x, self.end.x);
            max_axis = std::cmp::max(self.start.x, self.end.x);
        } else {
            panic!("invalid brick bounds");
        }

        let start = self.start;
        (min_axis..=max_axis).map(move |value| match axis {
            Axis::X => Position {
                x: value,
                y: start.y,
                z: start.z,
            },
            Axis::Y => Position {
                x: start.x,
                y: value,
                z: start.z,
            },
            Axis::Z => Position {
                x: start.x,
                y: start.y,
                z: value,
            },
        })
    }

    fn fall(&mut self) {
        self.start.z = std::cmp::max(self.start.z.saturating_sub(1), 1);
        self.end.z = std::cmp::max(self.end.z.saturating_sub(1), 1);
    }
}

impl std::str::FromStr for Brick {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s.split_once('~').ok_or_eyre("invalid brick")?;
        let start = start.parse()?;
        let end = end.parse()?;

        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Position {
    x: u32,
    y: u32,
    z: u32,
}

impl Position {
    fn above(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }

    fn below(&self) -> Option<Self> {
        if self.z > 1 {
            Some(Self {
                x: self.x,
                y: self.y,
                z: self.z - 1,
            })
        } else {
            None
        }
    }
}

impl std::str::FromStr for Position {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, s) = s.split_once(',').ok_or_eyre("invalid position")?;
        let (y, z) = s.split_once(',').ok_or_eyre("invalid position")?;

        let x = x.parse()?;
        let y = y.parse()?;
        let z = z.parse()?;

        Ok(Self { x, y, z })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BrickId(usize);

struct Space {
    bricks: Vec<Brick>,
    spaces: HashMap<Position, BrickId>,
}

impl Space {
    fn new(bricks: impl IntoIterator<Item = Brick>) -> Self {
        let bricks: Vec<_> = bricks.into_iter().collect();

        let mut spaces = HashMap::new();
        for (brick_id, brick) in bricks.iter().enumerate() {
            for pos in brick.positions() {
                let prev_space = spaces.insert(pos, BrickId(brick_id));
                assert!(
                    prev_space.is_none(),
                    "tried to insert multiple bricks in the same space"
                );
            }
        }

        Self { bricks, spaces }
    }

    fn settle_tick(&mut self) -> bool {
        for (brick_id, brick) in self.bricks.iter_mut().enumerate() {
            let can_fall = brick.positions().all(|pos| {
                if let Some(below) = pos.below() {
                    !self.spaces.contains_key(&below)
                } else {
                    false
                }
            });
            if can_fall {
                for pos in brick.positions() {
                    self.spaces.remove(&pos);
                }
                brick.fall();
                for pos in brick.positions() {
                    self.spaces.insert(pos, BrickId(brick_id));
                }

                return true;
            }
        }

        false
    }

    fn settle(&mut self) {
        while self.settle_tick() {}
    }

    fn num_disintegratable(&self) -> usize {
        let mut dependents: BTreeMap<BrickId, HashSet<BrickId>> = BTreeMap::new();

        for (brick_id, brick) in self.bricks.iter().enumerate() {
            for pos in brick.positions() {
                if let Some(dependent_id) = self.spaces.get(&pos.above()) {
                    dependents
                        .entry(*dependent_id)
                        .or_default()
                        .insert(BrickId(brick_id));
                }
            }
        }

        let non_disintegratable = dependents
            .iter()
            .filter_map(|(_, dependencies)| {
                if dependencies.len() == 1 {
                    dependencies.iter().next().copied()
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        self.bricks.len() - non_disintegratable.len()
    }
}

enum Axis {
    X,
    Y,
    Z,
}
