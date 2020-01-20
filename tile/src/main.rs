use csv::Reader as CSVReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct InputJSON {
    nodes: Vec<Point>,
    #[serde(flatten)]
    json: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InputCSV {
    lat: f64,
    lon: f64,
    price_adjusted_2019: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    #[serde(flatten)]
    json: InputJSON,
    #[serde(flatten)]
    csv: InputCSV,
}

fn main() {
    let json_file = File::open("../data/shared/joined.jsonl").unwrap();
    let json_reader = BufReader::new(json_file);
    let mut csv_reader = CSVReader::from_path("../data/shared/adjusted.csv").unwrap();

    fs::remove_dir_all("../data/tiles").unwrap_or_default();
    fs::create_dir("../data/tiles").unwrap();

    for (json_line, csv) in json_reader.lines().zip(csv_reader.deserialize()) {
        let json: InputJSON = serde_json::from_str(&json_line.unwrap()).unwrap();
        let csv: InputCSV = csv.unwrap();
        let lat = format!("{:0>5}", ((csv.lat + 90.) * 100.).trunc());
        let lon = format!("{:0>5}", ((csv.lon + 180.) * 100.).trunc());
        let path = format!("../data/tiles/{}{}.jsonl", lat, lon);
        let output = Output { json, csv };
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        let jsonl = serde_json::to_string(&output).unwrap();
        file.write(jsonl.as_bytes()).unwrap();
        file.write(b"\n").unwrap();
    }
}
