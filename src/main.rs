use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

struct Station {
    cnt: u32,
    sum: i64,
    min: i64,
    max: i64,
}

impl Station {
    fn new(value: i64) -> Station {
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

fn parse_num(s: &str) -> i64 {
    let (sign, val) = s
        .chars()
        .filter_map(|c| {
            if let Some(n) = c.to_digit(10) {
                Some((1i64, n as i64))
            } else if c == '-' {
                Some((-1, 0))
            } else {
                None
            }
        })
        .fold((1i64, 0i64), |(acc_sign, acc_sum), (sign, n)| {
            (sign * acc_sign, acc_sum * 10 + n)
        });
    sign * val
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

            (name.to_string(), parse_num(value))
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
    let slice_count = len / (10 * 1024 * 1024);
    let mut buf = [0u8; 256];

    let positions: Vec<u64> = (0..=slice_count)
        .map(|n| {
            let base_position = n * len / slice_count;
            if base_position == 0 || base_position == slice_count {
                return base_position;
            }

            f.seek(SeekFrom::Start(base_position)).unwrap();
            f.read(&mut buf).unwrap();
            base_position + (buf.iter().position(|c| *c == b'\n').unwrap() as u64) + 1
        })
        .collect();

    positions
        .windows(2)
        .map(|pair| match pair {
            [a, b] => (*a, *b - *a),
            _ => panic!(),
        })
        .collect()
}

fn merge_stats(
    mut a: HashMap<String, Station>,
    b: HashMap<String, Station>,
) -> HashMap<String, Station> {
    for (k, v) in b.into_iter() {
        if let Some(station) = a.get_mut(&k) {
            station.merge(&v);
        } else {
            a.insert(k, v);
        }
    }

    a
}

fn main() {
    use rayon::prelude::*;

    let src = args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "measurements.txt".to_string());

    let ranges = get_ranges(File::open(&src).unwrap());

    let stations = ranges
        .into_par_iter()
        .map(|(offset, len)| {
            let mut f = File::open(&src).unwrap();
            f.seek(SeekFrom::Start(offset)).unwrap();
            let subset = f.take(len);
            find_stats(subset)
        })
        .reduce_with(merge_stats)
        .unwrap();

    let mut keys: Vec<String> = stations.keys().map(|x| x.to_string()).collect();
    keys.sort();

    for x in keys.into_iter() {
        let station = &stations[&x];

        println!(
            "{}: {:.1} {:.1} {:.1} ({} samples)",
            x,
            station.min as f32 / 10.0,
            station.sum as f32 / (station.cnt as f32) / 10.0,
            station.max as f32 / 10.0,
            station.cnt
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_num() {
        assert_eq!(parse_num("123"), 123);
        assert_eq!(parse_num("-123"), -123);
        assert_eq!(parse_num("0"), 0);
        assert_eq!(parse_num("1.23"), 123);
        assert_eq!(parse_num("-1.23"), -123);
    }
}
