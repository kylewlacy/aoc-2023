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
        .without_time()
        .init();
    color_eyre::install()?;

    let mut stdin = std::io::stdin().lock();
    let mut input = String::new();
    stdin.read_to_string(&mut input)?;

    let mut modules: Modules = input.parse()?;
    let pulses = (0..1000)
        .flat_map(|_| modules.press_button())
        .collect::<Vec<_>>();

    let low_pulses = pulses
        .iter()
        .filter(|pulse| pulse.pulse == Pulse::Low)
        .count();
    let high_pulses = pulses
        .iter()
        .filter(|pulse| pulse.pulse == Pulse::High)
        .count();
    let value = low_pulses * high_pulses;
    println!("{value}");

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

trait Module {
    fn handle(&mut self, sent: &SentPulse) -> Option<Pulse>;
}

struct UntypedModule;

impl Module for UntypedModule {
    fn handle(&mut self, _sent: &SentPulse) -> Option<Pulse> {
        None
    }
}

#[derive(Default)]
struct FlipFlopModule {
    on: bool,
}

impl Module for FlipFlopModule {
    fn handle(&mut self, sent: &SentPulse) -> Option<Pulse> {
        match sent.pulse {
            Pulse::High => None,
            Pulse::Low => {
                if self.on {
                    self.on = false;
                    Some(Pulse::Low)
                } else {
                    self.on = true;
                    Some(Pulse::High)
                }
            }
        }
    }
}

#[derive(Default)]
struct ConjunctionModule {
    pulses: HashMap<String, Pulse>,
}

impl ConjunctionModule {
    fn new<'a>(sources: impl Iterator<Item = &'a str>) -> Self {
        Self {
            pulses: sources
                .map(|source| (source.to_string(), Pulse::Low))
                .collect(),
        }
    }
}

impl Module for ConjunctionModule {
    fn handle(&mut self, sent: &SentPulse) -> Option<Pulse> {
        self.pulses.insert(sent.source.clone(), sent.pulse);

        if self.pulses.values().all(|pulse| *pulse == Pulse::High) {
            Some(Pulse::Low)
        } else {
            Some(Pulse::High)
        }
    }
}

struct BroadcastModule;

impl Module for BroadcastModule {
    fn handle(&mut self, sent: &SentPulse) -> Option<Pulse> {
        Some(sent.pulse)
    }
}

struct Modules {
    modules: HashMap<String, Box<dyn Module>>,
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
                (name, ty.build(module_sources.iter().map(|s| &**s)))
            })
            .collect();

        Ok(Self {
            modules,
            destinations,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum ModuleType {
    Broadcast,
    FlipFlop,
    Conjunction,
    Untyped,
}

impl ModuleType {
    fn build<'a>(&self, sources: impl Iterator<Item = &'a str>) -> Box<dyn Module> {
        match self {
            ModuleType::Broadcast => Box::new(BroadcastModule),
            ModuleType::FlipFlop => Box::new(FlipFlopModule::default()),
            ModuleType::Conjunction => Box::new(ConjunctionModule::new(sources)),
            ModuleType::Untyped => Box::new(UntypedModule),
        }
    }
}
