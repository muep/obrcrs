use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::thread::spawn;

struct Station {
    cnt: u32,
    sum: f32,
    min: f32,
    max: f32,
}

impl Station {
    fn new(value: f32) -> Station {
        Station {
            cnt: 1,
            sum: value,
            min: value,
            max: value,
        }
    }

    fn merge(&mut self, other: &Station) {
        self.cnt += other.cnt;
        self.sum += other.sum;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
}

fn find_stats<T>(f: T) -> HashMap<String, Station>
where
    T: Read,
{
    let reader = BufReader::new(f);

    let mut stations: HashMap<String, Station> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let (name, value) = {
            let (name, value) = line.split_once(';').unwrap();

            (name.to_string(), value.parse::<f32>().unwrap())
        };

        if let Some(station) = stations.get_mut(&name) {
            station.cnt += 1;
            station.sum += value;
            station.min = station.min.min(value);
            station.max = station.max.max(value);
        } else {
            stations.insert(name, Station::new(value));
        }
    }

    stations
}

fn get_ranges(mut f: File) -> Vec<(u64, u64)> {
    let len = f.metadata().unwrap().len();
    let slice_count = std::thread::available_parallelism().unwrap().get() as u64;
    let mut buf = [0u8; 256];

    let positions: Vec<u64> = (0 ..= slice_count).map(|n| {
        let base_position = n * len / slice_count;
        if base_position == 0 || base_position == slice_count {
            return base_position;
        }

        f.seek(SeekFrom::Start(base_position)).unwrap();
        f.read(&mut buf).unwrap();
        base_position + (buf.iter().position(|c| *c == b'\n').unwrap() as u64) + 1
    }).collect();

    positions.windows(2).map(| pair| match pair {
        [a, b] => (*a, *b - *a),
        _ => panic!()
    }).collect()
}

fn merge_stats(mut a: HashMap<String, Station>, b: HashMap<String, Station>) -> HashMap<String, Station> {
    for (k,v) in b.into_iter() {
        if let Some(station) = a.get_mut(&k) {
            station.merge(&v);
        } else {
            a.insert(k, v);
        }
    }

    a
}

fn main() {
    let src = args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "measurements.txt".to_string());

    let ranges = get_ranges(File::open(&src).unwrap());

    let threads: Vec<_> = ranges.into_iter()
        .map(| (offset, len) | {
            let src = src.to_string();
            spawn(move || {
                let mut f = File::open(&src).unwrap();
                f.seek(SeekFrom::Start(offset)).unwrap();
                let subset = f.take(len);
                find_stats(subset)
            })
        })
        .collect();

    let stations = threads.into_iter()
        .map(|t| t.join().unwrap())
        .reduce(merge_stats)
        .unwrap();

    let mut keys: Vec<String> = stations.keys().map(|x| x.to_string()).collect();
    keys.sort();

    for x in keys.into_iter() {
        let station = &stations[&x];

        println!(
            "{}: {} {} {} ({} samples)",
            x,
            station.min,
            station.sum / (station.cnt as f32),
            station.max,
            station.cnt
        );
    }
}
