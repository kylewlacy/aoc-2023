use std::{
    collections::{HashMap, VecDeque},
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
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();
    color_eyre::install()?;

    let mut stdin = std::io::stdin().lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input)?;

    let mut modules: Modules = input.parse()?;

    let mut presses = 0;
    tracing::info!("starting");
    loop {
        let pulses = modules.press_button();
        presses += 1;
        if pulses
            .iter()
            .any(|pulse| pulse.destination == "rx" && pulse.pulse == Pulse::Low)
        {
            break;
        }
    }
    tracing::info!("finished");
    println!("{presses}");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pulse {
    Low,
    High,
}

#[derive(Debug, Clone)]
struct SentPulse {
    pulse: Pulse,
    source: String,
    destination: String,
}

impl std::fmt::Display for SentPulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pulse = match self.pulse {
            Pulse::Low => "low",
            Pulse::High => "high",
        };
        write!(f, "{} -{pulse}-> {}", self.source, self.destination)
    }
}

#[derive(Debug, Clone, Copy)]
enum ModuleType {
    Broadcast,
    FlipFlop,
    Conjunction,
    Untyped,
}

enum Module {
    Untyped,
    FlipFlop { on: bool },
    Conjunction { pulses: HashMap<String, Pulse> },
    Broadcast,
}

impl Module {
    fn new<'a>(ty: ModuleType, sources: impl Iterator<Item = &'a str>) -> Self {
        match ty {
            ModuleType::Broadcast => Self::Broadcast,
            ModuleType::FlipFlop => Self::FlipFlop { on: false },
            ModuleType::Conjunction => Self::Conjunction {
                pulses: sources
                    .map(|source| (source.to_string(), Pulse::Low))
                    .collect(),
            },
            ModuleType::Untyped => Self::Untyped,
        }
    }

    fn handle(&mut self, sent: &SentPulse) -> Option<Pulse> {
        match self {
            Module::Untyped => None,
            Module::FlipFlop { on } => match sent.pulse {
                Pulse::High => None,
                Pulse::Low => {
                    if *on {
                        *on = false;
                        Some(Pulse::Low)
                    } else {
                        *on = true;
                        Some(Pulse::High)
                    }
                }
            },
            Module::Conjunction { pulses } => {
                pulses.insert(sent.source.clone(), sent.pulse);

                if pulses.values().all(|pulse| *pulse == Pulse::High) {
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            }
            Module::Broadcast => Some(sent.pulse),
        }
    }
}

struct Modules {
    modules: HashMap<String, Module>,
    destinations: HashMap<String, Vec<String>>,
}

impl Modules {
    fn press_button(&mut self) -> Vec<SentPulse> {
        let mut handled_pulses = vec![];
        let mut unhandled_pulses: VecDeque<_> = [SentPulse {
            source: "button".to_string(),
            destination: "broadcaster".to_string(),
            pulse: Pulse::Low,
        }]
        .into_iter()
        .collect();

        while let Some(pulse) = unhandled_pulses.pop_front() {
            tracing::debug!("handling pulse: {pulse}");
            handled_pulses.push(pulse.clone());

            let pulse_module = &pulse.destination;
            if let Some(module) = self.modules.get_mut(pulse_module) {
                if let Some(next_pulse) = module.handle(&pulse) {
                    tracing::debug!("- module {pulse_module} generated pulse {next_pulse:?}");
                    for dest in &self.destinations[&pulse.destination] {
                        tracing::debug!("- sending {pulse_module} pulse {next_pulse:?} to {dest}");
                        unhandled_pulses.push_back(SentPulse {
                            source: pulse_module.to_string(),
                            destination: dest.to_string(),
                            pulse: next_pulse,
                        });
                    }
                }
            } else {
                tracing::debug!("- module {pulse_module} doesn't exist, skipping");
            }
        }

        handled_pulses
    }
}

impl std::str::FromStr for Modules {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut module_types = HashMap::new();
        let mut destinations: HashMap<String, Vec<String>> = HashMap::new();

        for line in s.lines() {
            let (label, destination_list) = line.split_once(" -> ").ok_or_eyre("invalid module")?;
            let name;
            let module_type;
            if let Some(label_name) = label.strip_prefix('%') {
                name = label_name;
                module_type = ModuleType::FlipFlop;
            } else if let Some(label_name) = label.strip_prefix('&') {
                name = label_name;
                module_type = ModuleType::Conjunction;
            } else if label == "broadcaster" {
                name = label;
                module_type = ModuleType::Broadcast;
            } else {
                name = label;
                module_type = ModuleType::Untyped;
            }

            module_types.insert(name.to_string(), module_type);
            destinations.insert(
                name.to_string(),
                destination_list.split(", ").map(Into::into).collect(),
            );

            for dest in destination_list.split(", ") {
                module_types
                    .entry(dest.to_string())
                    .or_insert(ModuleType::Untyped);
            }
        }

        let mut sources: HashMap<String, Vec<String>> = module_types
            .iter()
            .map(|(name, _)| (name.to_string(), vec![]))
            .collect();
        for (source, dests) in &destinations {
            for dest in dests {
                let dest_sources = sources.entry(dest.to_string()).or_default();
                dest_sources.push(source.to_string());
            }
        }

        let modules = module_types
            .into_iter()
            .map(|(name, ty)| {
                let module_sources = &sources[&name];
                (name, Module::new(ty, module_sources.iter().map(|s| &**s)))
            })
            .collect();

        Ok(Self {
            modules,
            destinations,
        })
    }
}
