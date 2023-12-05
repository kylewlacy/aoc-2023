#![feature(btree_cursors)]

use std::{collections::BTreeMap, io::Read, ops::Bound};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .without_time()
        .init();
    color_eyre::install()?;

    let mut input = String::new();
    let mut stdin = std::io::stdin().lock();
    stdin.read_to_string(&mut input)?;

    let almanac: Almanac = input.parse()?;

    let min_seed_location = almanac
        .seeds
        .iter()
        .map(|&seed| almanac.seed_location(seed))
        .min()
        .ok_or_else(|| eyre::eyre!("no seeds"))?;

    println!("{min_seed_location}");

    Ok(())
}

struct Almanac {
    seeds: Vec<u32>,
    seed_to_soil_map: RangeMap,
    soil_to_fertilizer_map: RangeMap,
    fertilizer_to_water_map: RangeMap,
    water_to_light_map: RangeMap,
    light_to_temperature_map: RangeMap,
    temperature_to_humidity_map: RangeMap,
    humidity_to_location_map: RangeMap,
}

impl Almanac {
    fn seed_location(&self, seed: u32) -> u32 {
        let soil = self.seed_to_soil_map.get(seed);
        let fertilizer = self.soil_to_fertilizer_map.get(soil);
        let water = self.fertilizer_to_water_map.get(fertilizer);
        let light = self.water_to_light_map.get(water);
        let temperature = self.light_to_temperature_map.get(light);
        let humidity = self.temperature_to_humidity_map.get(temperature);
        let location = self.humidity_to_location_map.get(humidity);
        tracing::debug!(
            ?seed,
            ?soil,
            ?fertilizer,
            ?water,
            ?light,
            ?temperature,
            ?humidity,
            ?location,
            "mapped seed",
        );

        location
    }
}

impl std::str::FromStr for Almanac {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections = s.split("\n\n");

        let seeds_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let seed_list = seeds_section
            .strip_prefix("seeds: ")
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let seeds = seed_list
            .split_whitespace()
            .map(|seed| Ok(seed.parse()?))
            .collect::<eyre::Result<Vec<_>>>()?;

        let seed_to_soil_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let seed_to_soil_map =
            parse_range_map_section(seed_to_soil_section, "seed-to-soil map:\n")?;

        let soil_to_fertilizer_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let soil_to_fertilizer_map =
            parse_range_map_section(soil_to_fertilizer_section, "soil-to-fertilizer map:\n")?;

        let fertilizer_to_water_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let fertilizer_to_water_map =
            parse_range_map_section(fertilizer_to_water_section, "fertilizer-to-water map:\n")?;

        let water_to_light_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let water_to_light_map =
            parse_range_map_section(water_to_light_section, "water-to-light map:\n")?;

        let light_to_temperature_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let light_to_temperature_map =
            parse_range_map_section(light_to_temperature_section, "light-to-temperature map:\n")?;

        let temperature_to_humidity_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let temperature_to_humidity_map = parse_range_map_section(
            temperature_to_humidity_section,
            "temperature-to-humidity map:\n",
        )?;

        let humidity_to_location_section = sections
            .next()
            .ok_or_else(|| eyre::eyre!("invalid almanac"))?;
        let humidity_to_location_map =
            parse_range_map_section(humidity_to_location_section, "humidity-to-location map:\n")?;

        Ok(Self {
            seeds,
            seed_to_soil_map,
            soil_to_fertilizer_map,
            fertilizer_to_water_map,
            water_to_light_map,
            light_to_temperature_map,
            temperature_to_humidity_map,
            humidity_to_location_map,
        })
    }
}

fn parse_range_map_section(section: &str, prefix: &str) -> eyre::Result<RangeMap> {
    let seed_to_soil_entries = section
        .strip_prefix(prefix)
        .ok_or_else(|| eyre::eyre!("section title did not match"))?;
    let mut map = RangeMap::new();
    for entry in seed_to_soil_entries.lines() {
        let mut values = entry.split_whitespace();
        let from_start = values.next().ok_or_else(|| eyre::eyre!("invalid record"))?;
        let to_start = values.next().ok_or_else(|| eyre::eyre!("invalid record"))?;
        let length = values.next().ok_or_else(|| eyre::eyre!("invalid record"))?;

        let destination_start = from_start.parse()?;
        let source_start = to_start.parse()?;
        let length = length.parse()?;

        map.add_range(destination_start, source_start, length);
    }

    Ok(map)
}

#[derive(Debug)]
struct RangeMap {
    entries: BTreeMap<u32, (u32, u32)>,
}

impl RangeMap {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    fn add_range(&mut self, destination_start: u32, source_start: u32, length: u32) {
        self.entries
            .insert(source_start, (destination_start, length));
    }

    fn get(&self, key: u32) -> u32 {
        let cursor = self.entries.upper_bound(Bound::Included(&key));
        cursor
            .key_value()
            .and_then(|(source_start, (dest_start, length))| {
                let offset = key.checked_sub(*source_start).unwrap();
                if offset < *length {
                    Some(dest_start + offset)
                } else {
                    None
                }
            })
            .unwrap_or(key)
    }
}
