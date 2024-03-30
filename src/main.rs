use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct Station {
    count: u32,
    sum: f32,
    min: f32,
    max: f32,
}

impl Station {
    fn new(value: f32) -> Station {
        Station {
            count: 1,
            sum: value,
            min: value,
            max: value,
        }
    }
}

fn main() {
    let src = args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "measurements.txt".to_string());

    let f = File::open(&src).unwrap();
    let reader = BufReader::new(f);

    let mut stations: HashMap<String, Station> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let (name, value) = {
            let (name, value) = line.split_once(';').unwrap();

            (name.to_string(), value.parse::<f32>().unwrap())
        };

        if let Some(station) = stations.get(&name) {
            stations.insert(
                name,
                Station {
                    count: station.count + 1,
                    sum: station.sum + value,
                    min: station.min.min(value),
                    max: station.max.max(value),
                },
            );
        } else {
            stations.insert(name, Station::new(value));
        }
    }

    let mut keys: Vec<String> = stations.keys().map(|x| x.to_string()).collect();
    keys.sort();

    for x in keys.into_iter() {
        let station = &stations[&x];

        println!(
            "{}: {} {} {} ({} samples)",
            x,
            station.min,
            station.sum / (station.count as f32),
            station.max,
            station.count
        );
    }
}
