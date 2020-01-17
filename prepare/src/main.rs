use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Input {
    nodes: Vec<Point>,
    price_paid: String,
    transaction_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    lat: f64,
    lon: f64,
    price_paid: u32,
    transaction_at: String,
}

fn main() {
    let file = File::open("../data/shared/joined.jsonl").unwrap();
    let reader = BufReader::new(file);

    fs::remove_file("../data/shared/prepared.csv").unwrap_or_default();
    let file = File::create("../data/shared/prepared.csv").unwrap();
    let mut writer = csv::Writer::from_writer(file);

    for line in reader.lines() {
        let i: Input = serde_json::from_str(&line.unwrap()).unwrap();
        let p = avg_points(i.nodes);
        let o = Output {
            lat: p.lat,
            lon: p.lon,
            price_paid: i.price_paid.parse().unwrap(),
            transaction_at: i.transaction_at,
        };
        writer.serialize(o).unwrap();
    }
}

fn avg_points(p: Vec<Point>) -> Point {
    let lat: f64 = p.iter().map(|p| p.lat).sum();
    let lon: f64 = p.iter().map(|p| p.lon).sum();

    let len = p.len() as f64;
    let lat = lat / len;
    let lon = lon / len;

    Point { lat, lon }
}
