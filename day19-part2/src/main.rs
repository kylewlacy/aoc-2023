use std::{collections::HashMap, io::Read as _, ops::RangeInclusive};

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

    let Some((workflows, _parts)) = input.split_once("\n\n") else {
        eyre::bail!("invalid input");
    };
    let workflows = Workflows::parse(workflows)?;

    let rules = workflows.to_rule();
    let total_possible_parts: u64 = rules
        .filter_parts(&PartSet::all())
        .into_iter()
        .map(|parts| parts.len())
        .sum();

    println!("{:#?}", total_possible_parts);

    // let value: i64 = parts
    //     .iter()
    //     .filter(|part| workflows.eval(part))
    //     .map(|part| part.value())
    //     .sum();
    // println!("{value}");

    Ok(())
}

#[derive(Debug, Clone)]
struct Workflows {
    workflows: HashMap<String, Workflow>,
}

impl Workflows {
    fn parse(s: &str) -> eyre::Result<Self> {
        let workflows = s
            .lines()
            .map(|line| {
                let workflow: Workflow = line.parse()?;
                eyre::Ok((workflow.name.clone(), workflow))
            })
            .collect::<eyre::Result<_>>()?;

        Ok(Self { workflows })
    }

    fn to_rule(&self) -> PartRule {
        self.workflow_to_rule("in")
    }

    fn workflow_to_rule(&self, workflow_name: &str) -> PartRule {
        let workflow = &self.workflows[workflow_name];

        self.convert_rule(&workflow.rule)
    }

    fn convert_rule(&self, rule: &Rule) -> PartRule {
        match rule {
            Rule::Accept => PartRule::Accept,
            Rule::Reject => PartRule::Reject,
            Rule::Call(workflow) => self.workflow_to_rule(workflow),
            Rule::If {
                condition,
                then,
                else_,
            } => PartRule::If {
                condition: condition.clone(),
                then: Box::new(self.convert_rule(then)),
                else_: Box::new(self.convert_rule(else_)),
            },
        }
    }
}

#[derive(Debug, Clone)]
struct Workflow {
    name: String,
    rule: Rule,
}

impl std::str::FromStr for Workflow {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, s) = s.split_once('{').ok_or_eyre("invalid workflow")?;
        let rule = s.strip_suffix('}').ok_or_eyre("invalid workflow")?;
        let rule: Rule = rule.parse()?;

        Ok(Self {
            name: name.to_string(),
            rule,
        })
    }
}

#[derive(Debug, Clone)]
enum Rule {
    Accept,
    Reject,
    Call(String),
    If {
        condition: Condition,
        then: Box<Rule>,
        else_: Box<Rule>,
    },
}

impl std::str::FromStr for Rule {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let if_ = s
            .split_once(':')
            .and_then(|(cond, s)| s.split_once(',').map(|(then, else_)| (cond, then, else_)));

        if let Some((condition, then, else_)) = if_ {
            let condition = condition.parse()?;
            let then = then.parse()?;
            let else_ = else_.parse()?;

            return Ok(Self::If {
                condition,
                then: Box::new(then),
                else_: Box::new(else_),
            });
        }

        match s {
            "A" => Ok(Self::Accept),
            "R" => Ok(Self::Reject),
            workflow => Ok(Self::Call(workflow.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Condition {
    var: Var,
    comparison: Comparison,
    value: u16,
}

impl std::str::FromStr for Condition {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (var, s) = s.split_at(1);
        let (comparison, value) = s.split_at(1);

        let var = var.parse()?;
        let comparison = comparison.parse()?;
        let value = value.parse()?;

        Ok(Self {
            var,
            comparison,
            value,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Var {
    X,
    M,
    A,
    S,
}

impl std::str::FromStr for Var {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(Self::X),
            "m" => Ok(Self::M),
            "a" => Ok(Self::A),
            "s" => Ok(Self::S),
            other => {
                eyre::bail!("invalid var: {other:?}");
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Comparison {
    Gt,
    Lt,
}

impl std::str::FromStr for Comparison {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" => Ok(Self::Gt),
            "<" => Ok(Self::Lt),
            other => {
                eyre::bail!("invalid comparison: {other:?}");
            }
        }
    }
}

#[derive(Debug, Clone)]
enum PartRule {
    If {
        condition: Condition,
        then: Box<PartRule>,
        else_: Box<PartRule>,
    },
    Accept,
    Reject,
}

impl PartRule {
    fn filter_parts(&self, parts: &PartSet) -> Vec<PartSet> {
        match self {
            PartRule::If {
                condition,
                then,
                else_,
            } => {
                let (then_set, else_set) = parts.split(*condition);
                let mut parts = then.filter_parts(&then_set);
                parts.extend_from_slice(&else_.filter_parts(&else_set));

                parts
            }
            PartRule::Accept => vec![parts.clone()],
            PartRule::Reject => vec![PartSet::none()],
        }
    }
}

#[derive(Debug, Clone)]
struct PartSet {
    x: RangeInclusive<u16>,
    m: RangeInclusive<u16>,
    a: RangeInclusive<u16>,
    s: RangeInclusive<u16>,
}

impl PartSet {
    fn all() -> Self {
        PartSet {
            x: 1..=4000,
            m: 1..=4000,
            a: 1..=4000,
            s: 1..=4000,
        }
    }

    fn none() -> Self {
        PartSet {
            x: 1..=0,
            m: 1..=0,
            a: 1..=0,
            s: 1..=0,
        }
    }

    fn len(&self) -> u64 {
        self.x.len() as u64 * self.m.len() as u64 * self.a.len() as u64 * self.s.len() as u64
    }

    fn range(&self, var: Var) -> &RangeInclusive<u16> {
        match var {
            Var::X => &self.x,
            Var::M => &self.m,
            Var::A => &self.a,
            Var::S => &self.s,
        }
    }

    fn with_range(&self, var: Var, range: RangeInclusive<u16>) -> Self {
        let Self { x, m, a, s } = self.clone();
        match var {
            Var::X => Self { x: range, m, a, s },
            Var::M => Self { x, m: range, a, s },
            Var::A => Self { x, m, a: range, s },
            Var::S => Self { x, m, a, s: range },
        }
    }

    fn split(&self, condition: Condition) -> (Self, Self) {
        let range = self.range(condition.var);
        let (then, else_) = split_range(&range, condition.comparison, condition.value);
        (
            self.with_range(condition.var, then),
            self.with_range(condition.var, else_),
        )
    }
}

fn split_range(
    range: &RangeInclusive<u16>,
    comparison: Comparison,
    value: u16,
) -> (RangeInclusive<u16>, RangeInclusive<u16>) {
    match comparison {
        Comparison::Gt => (value + 1..=*range.end(), *range.start()..=value),
        Comparison::Lt if value == 0 => (1..=0, range.clone()),
        Comparison::Lt => (*range.start()..=value - 1, value..=*range.end()),
    }
}
