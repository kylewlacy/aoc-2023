use std::{collections::HashMap, io::Read as _};

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

    let Some((workflows, parts)) = input.split_once("\n\n") else {
        eyre::bail!("invalid input");
    };
    let workflows = Workflows::parse(workflows)?;
    let parts = parts
        .lines()
        .map(|line| line.parse())
        .collect::<eyre::Result<Vec<Part>>>()?;

    let value: i64 = parts
        .iter()
        .filter(|part| workflows.eval(part))
        .map(|part| part.value())
        .sum();
    println!("{value}");

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

    fn eval(&self, part: &Part) -> bool {
        self.eval_workflow("in", part)
    }

    fn eval_workflow(&self, workflow_name: &str, part: &Part) -> bool {
        let workflow = &self.workflows[workflow_name];

        match workflow.rule.eval(part) {
            RuleResult::Accept => true,
            RuleResult::Reject => false,
            RuleResult::Call(next_workflow) => self.eval_workflow(&next_workflow, part),
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

impl Rule {
    fn eval(&self, part: &Part) -> RuleResult {
        match self {
            Rule::Accept => RuleResult::Accept,
            Rule::Reject => RuleResult::Reject,
            Rule::Call(workflow) => RuleResult::Call(workflow.clone()),
            Rule::If {
                condition,
                then,
                else_,
            } => {
                if condition.eval(part) {
                    then.eval(part)
                } else {
                    else_.eval(part)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum RuleResult {
    Accept,
    Reject,
    Call(String),
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
    value: i64,
}

impl Condition {
    fn eval(&self, part: &Part) -> bool {
        let var_value = match self.var {
            Var::X => part.x,
            Var::M => part.m,
            Var::A => part.a,
            Var::S => part.s,
        };

        match self.comparison {
            Comparison::Gt => var_value > self.value,
            Comparison::Lt => var_value < self.value,
        }
    }
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

#[derive(Debug, Clone, Copy)]
struct Part {
    x: i64,
    m: i64,
    a: i64,
    s: i64,
}

impl Part {
    fn value(&self) -> i64 {
        self.x + self.m + self.a + self.s
    }
}

impl std::str::FromStr for Part {
    type Err = eyre::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let string = string.strip_prefix("{x=").ok_or_eyre("invalid part")?;
        let (x, string) = string.split_once(",m=").ok_or_eyre("invalid part")?;
        let (m, string) = string.split_once(",a=").ok_or_eyre("invalid part")?;
        let (a, string) = string.split_once(",s=").ok_or_eyre("invalid part")?;
        let s = string.strip_suffix('}').ok_or_eyre("invalid part")?;

        let x = x.parse()?;
        let m = m.parse()?;
        let a = a.parse()?;
        let s = s.parse()?;

        Ok(Self { x, m, a, s })
    }
}
