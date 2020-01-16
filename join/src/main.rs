use csv::ByteRecord;
use csv::ReaderBuilder as CSVReader;
use kdtree::{distance::squared_euclidean, KdTree};
use quick_xml::events as xml_events;
use quick_xml::events::Event as XMLEvent;
use quick_xml::Reader as XMLReader;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Point {
  lat: f64,
  lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct House {
  nodes: Vec<Point>,
  #[serde(flatten)]
  attrs: HashMap<String, String>,
}

#[derive(StructOpt, Debug)]
struct Cli {
  #[structopt(long = "pp", parse(from_os_str))]
  pp: PathBuf,
  #[structopt(long = "osm", parse(from_os_str), multiple = true)]
  osm: Vec<PathBuf>,
  #[structopt(long = "postcode", parse(from_os_str))]
  postcode: PathBuf,
  #[structopt(short = "o", long = "output", parse(from_os_str))]
  output: PathBuf,
}

// HM<K, A>, I<K, B> => (A, B, K) for every HM(K) == I(K)
// HM memory / I time
// HM clones values
// HM itself is borrowed, so HM can match multiple I in its lifetime
struct HashMatchIterator<'a, A: Clone, B, K: Hash + Eq, I: Iterator<Item = (K, B)>> {
  hashmap: &'a HashMap<K, A>,
  iter: I,
}

impl<'a, A: Clone, B, K: Hash + Eq, I: Iterator<Item = (K, B)>> Iterator
  for HashMatchIterator<'a, A, B, K, I>
{
  type Item = (A, B, K);

  fn next(&mut self) -> Option<Self::Item> {
    for (bk, bv) in self.iter.by_ref() {
      let av = self.hashmap.get(&bk);
      if let Some(av) = av {
        return Some((av.clone(), bv, bk));
      }
    }

    None
  }
}

fn main() {
  let args = Cli::from_args();

  let pp = args.pp;
  let osm = args.osm;
  let postcode = args.postcode;
  let output = args.output;

  osm.par_iter().for_each(|osm| {
    let mut osm = osm.clone();
    osm.set_extension("osm");
    let mut jsonl = output.clone();
    jsonl.push(osm.file_name().unwrap());
    jsonl.set_extension("jsonl");
    process_osm(
      pp.to_str().unwrap(),
      osm.to_str().unwrap(),
      postcode.to_str().unwrap(),
      jsonl.to_str().unwrap(),
    );
  })
}

fn process_osm(pp_path: &str, osm_path: &str, postcode_path: &str, result_path: &str) {
  fs::remove_file(&result_path).unwrap_or_default();
  let mut result_file = File::create(&result_path).unwrap();

  println!("opening osm... '{}'", &osm_path);
  let mut osm_file = File::open(&osm_path).unwrap();
  println!("getting houseseeks... '{}'", &osm_path);
  let mut houseseeks = get_houseseeks(&osm_path);
  println!("getting houserows... '{}'", &pp_path);
  let houserows = get_houserows(&pp_path);
  println!("getting nodepoints... '{}'", &osm_path);
  let nodepoints = get_nodepoints(&osm_path);
  println!("getting postcodes... '{}'", &postcode_path);
  let postcodes = get_postcodes(&postcode_path);

  println!("matching... run 'tail -f {}' for progress", &result_path);

  let hmi = HashMatchIterator {
    hashmap: &mut houseseeks,
    iter: houserows,
  };
  for (houseseek, houserow, _) in hmi {
    osm_file.seek(SeekFrom::Start(houseseek as u64)).unwrap();

    let (osm_attrs, nodes) = xml_seek_to_attrs(&osm_file, &nodepoints);

    let pp_postcode = &houserow[3];
    let osm_xy = [nodes[0].lat, nodes[0].lon];
    let osm_closest_postcodes: Vec<&[u8]> = postcodes
      .nearest(&osm_xy, 8, &squared_euclidean)
      .unwrap()
      .iter()
      .map(|p| &p.1[..])
      .collect();

    if !osm_closest_postcodes.contains(&pp_postcode) {
      continue;
    }

    let pp_attrs = csv_record_to_attrs(houserow);
    let mut attrs = osm_attrs;
    attrs.extend(pp_attrs.into_iter());

    let house = House { attrs, nodes };

    let jsonl = serde_json::to_string(&house).unwrap();
    result_file.write(jsonl.as_bytes()).unwrap();
    result_file.write(b"\n").unwrap();
  }

  println!("finished... '{}'", &result_path);
}

// csv -> ("[street]:[housenumber]", Record)
fn get_houserows(path: &str) -> impl Iterator<Item = (String, ByteRecord)> {
  let file = File::open(path).unwrap();
  let reader = CSVReader::new().has_headers(false).from_reader(file);

  reader.into_byte_records().map(|record| {
    let record = record.unwrap();
    let street = &record[9];
    let houseid = &record[7];

    let id = norm_houseid(
      str::from_utf8(street).unwrap(),
      str::from_utf8(houseid).unwrap(),
    );

    (id, record)
  })
}

// returns buffer position of potential houses while ignoring fences, statues, etc
// "[street]:[housenumber]" -> xml_buffer_position
fn get_houseseeks(path: &str) -> HashMap<String, usize> {
  let mut reader = XMLReader::from_file(path).unwrap();
  let mut houseseeks = HashMap::new();

  let mut is_houseseek = false;
  let mut houseseek = 0;
  let mut buf = Vec::new();

  let mut housename = Vec::new();
  let mut housenumber = Vec::new();
  let mut street = Vec::new();
  loop {
    let e = reader.read_event(&mut buf).unwrap();

    match e {
      // ex: <node>
      XMLEvent::Start(ref e) if e.name() == b"node" || e.name() == b"way" => {
        // houseseek is the buffer_position from immediately before an XMLEvent::Start where the tag
        // contains an address. is_houseseek prevents updates to houseseek when inside a potential match
        is_houseseek = true;
      }
      // ex: <tag k="addr:housename" v="The Royal Foresters" />
      XMLEvent::Empty(ref e) if e.name() == b"tag" && is_houseseek => {
        let (k, v) = get_tag_kv(e).unwrap();
        match &k[..] {
          b"addr:housename" if housename.is_empty() => housename.extend(v),
          b"addr:housenumber" if housenumber.is_empty() => housenumber.extend(v),
          b"addr:street" if street.is_empty() => street.extend(v),
          _ => (),
        }
      }
      // ex: </node>
      XMLEvent::End(ref e) if e.name() == b"node" || e.name() == b"way" => {
        if !housename.is_empty() && !street.is_empty() {
          let id = norm_houseid(
            str::from_utf8(&street).unwrap(),
            str::from_utf8(&housename).unwrap(),
          );
          houseseeks.insert(id, houseseek);
        }
        if !housenumber.is_empty() && !street.is_empty() {
          let id = norm_houseid(
            str::from_utf8(&street).unwrap(),
            str::from_utf8(&housenumber).unwrap(),
          );
          houseseeks.insert(id, houseseek);
        }
        is_houseseek = false;
        housename.clear();
        housenumber.clear();
        street.clear();
      }
      XMLEvent::Eof => {
        break;
      }
      _ => (),
    }

    if !is_houseseek {
      houseseek = reader.buffer_position();
    }

    buf.clear();
  }

  houseseeks
}

fn norm_houseid(street: &str, houseid: &str) -> String {
  let street = remove_entities(&street);
  let street = remove_punctuation(&street);
  let street = street.to_ascii_lowercase();
  let houseid = remove_entities(&houseid);
  let houseid = remove_punctuation(&houseid);
  let houseid = houseid.to_ascii_lowercase();
  let mut fullid = street;
  fullid.push(':');
  fullid += &houseid;
  fullid
}

// ex: "Tom &amp; Jerry" => "Tom Jerry"
fn remove_entities(s: &str) -> String {
  let mut amp = false;
  let mut next = String::new();
  let mut buf = String::new();
  for c in s.chars() {
    if c == '&' {
      amp = true;
    }
    if !amp {
      next.push(c);
    } else {
      buf.push(c);
    }
    if c == ';' {
      amp = false;
      buf.clear();
    }
  }
  next += &buf;

  next
}

fn remove_punctuation(s: &str) -> String {
  s.chars()
    .filter(|&c| c.is_alphanumeric() || c == ' ')
    .collect()
}

// KDTree is fast at getting nearest X postcodes to a point
fn get_postcodes(path: &str) -> KdTree<f64, Vec<u8>, Vec<f64>> {
  let dims = 2;
  let mut tree = KdTree::new(dims);

  let file = File::open(path).unwrap();
  let reader = CSVReader::new().from_reader(file);

  for record in reader.into_byte_records() {
    let record = record.unwrap();
    let postcode = Vec::from(&record[1]);
    let lat = str::from_utf8(&record[2])
      .unwrap()
      .parse()
      .unwrap_or_default();
    let lon = str::from_utf8(&record[3])
      .unwrap()
      .parse()
      .unwrap_or_default();
    tree.add(vec![lat, lon], postcode).unwrap_or_default();
  }

  tree
}

// sometimes houses store their location inline like:
//   <node id="218671" lat="53.3636318117118" lon="-2.1556549352908">
//
// sometimes they are instead like:
//   <way>
//     <nd ref="218671" />
//
// for refs, we get the house's location via nodepoints (ref -> Point)
fn get_nodepoints(path: &str) -> HashMap<Vec<u8>, Point> {
  let mut reader = XMLReader::from_file(path).unwrap();
  let mut nodepoints = HashMap::new();

  let mut buf = Vec::new();
  loop {
    let e = reader.read_event(&mut buf).unwrap();
    match e {
      XMLEvent::Start(ref e) | XMLEvent::Empty(ref e) if e.name() == b"node" => {
        if let Some((id, point)) = get_node_point(e) {
          nodepoints.insert(id, point);
        }
      }
      XMLEvent::Eof => {
        break;
      }
      _ => (),
    }

    buf.clear();
  }

  nodepoints
}

// <node id="218671" lat="53.3636318117118" lon="-2.1556549352908">
fn get_node_point(e: &xml_events::BytesStart) -> Option<(Vec<u8>, Point)> {
  let mut id = Vec::new();
  let mut lat = 0.;
  let mut lon = 0.;
  for a in e.attributes() {
    let a = a.unwrap();
    match a.key {
      b"id" => id.extend_from_slice(&a.value),
      b"lat" => lat = str::from_utf8(&a.value).unwrap().parse().unwrap(),
      b"lon" => lon = str::from_utf8(&a.value).unwrap().parse().unwrap(),
      _ => (),
    }
  }
  if id.is_empty() || lat == 0. || lon == 0. {
    None
  } else {
    Some((id, Point { lat, lon }))
  }
}

// <tag k="addr:city" v="Reading" />
fn get_tag_kv(e: &xml_events::BytesStart) -> Option<(Vec<u8>, Vec<u8>)> {
  let mut k = Vec::new();
  let mut v = Vec::new();
  for a in e.attributes() {
    let a = a.unwrap();
    match a.key {
      b"k" => k.extend_from_slice(&a.value),
      b"v" => v.extend_from_slice(&a.value),
      _ => (),
    }
  }
  if k.is_empty() || v.is_empty() {
    None
  } else {
    Some((k, v))
  }
}

// only gets one attr at a time
fn get_xml_attr(e: &xml_events::BytesStart, k: &[u8]) -> Option<Vec<u8>> {
  for a in e.attributes() {
    let a = a.unwrap();
    if a.key == k {
      return Some(Vec::from(a.value));
    }
  }
  None
}

// osm "business logic"
fn xml_seek_to_attrs(
  file: &File,
  nodepoints: &HashMap<Vec<u8>, Point>,
) -> (HashMap<String, String>, Vec<Point>) {
  let mut attrs = HashMap::new();
  let mut nodes = Vec::new();
  let mut node_refs = Vec::new();

  let reader = BufReader::new(file);
  let mut reader = XMLReader::from_reader(reader);

  let mut buf = Vec::new();
  loop {
    let e = reader.read_event(&mut buf).unwrap();

    match e {
      // get lat/lon from <node lat="53.3636318117118" lon="-2.1556549352908">
      XMLEvent::Start(ref e) if e.name() == b"node" => {
        if let Some((_, point)) = get_node_point(e) {
          nodes.push(point);
        }
      }
      // eventually get lat/lon from "ref" as in <way><nd ref="218671" />...</way>
      XMLEvent::Empty(ref e) if e.name() == b"nd" => {
        if let Some(id) = get_xml_attr(e, b"ref") {
          node_refs.push(id);
        }
      }
      // <tag k="addr:city" v="Reading" />
      XMLEvent::Empty(ref e) if e.name() == b"tag" => {
        if let Some((k, v)) = get_tag_kv(e) {
          attrs.insert(String::from_utf8(k).unwrap(), String::from_utf8(v).unwrap());
        }
      }
      // xml_seek_to_attrs only processes one xml node/way
      XMLEvent::End(ref e) if e.name() == b"node" || e.name() == b"way" => {
        break;
      }
      XMLEvent::Eof => {
        break;
      }
      _ => (),
    }

    buf.clear();
  }

  let npi = HashMatchIterator {
    hashmap: &nodepoints,
    iter: node_refs.into_iter().map(|id| (id, None::<bool>)),
  };
  for (point, _, _) in npi {
    nodes.push(point);
  }

  (attrs, nodes)
}

// pp "business logic"
fn csv_record_to_attrs(record: csv::ByteRecord) -> HashMap<String, String> {
  let mut attrs = HashMap::new();

  for (i, v) in record.iter().enumerate() {
    let mut v = v;
    let k = match i {
      0 => Some("id"),
      1 => Some("price_paid"),
      2 => Some("transaction_at"),
      3 => Some("addr:postcode"),
      4 => {
        match v {
          b"D" => v = b"Detached",
          b"S" => v = b"Semi-Detached",
          b"T" => v = b"Terraced",
          b"F" => v = b"Flat / Maisonette",
          b"O" => v = b"Other",
          _ => (),
        };
        Some("property_type")
      }
      5 => {
        match v {
          b"Y" => v = b"Yes",
          _ => v = b"No",
        };
        Some("new_build")
      }
      6 => {
        match v {
          b"F" => v = b"Yes",
          _ => v = b"No",
        };
        Some("freehold")
      }
      7 => None, // "POAN" comes from osm
      8 => Some("addr:secondary"),
      9 => None, // "addr:street" comes from osm
      10 => Some("addr:locality"),
      11 => Some("addr:city"),
      12 => Some("addr:district"),
      13 => Some("addr:county"),
      14 => None, // skip price paid category type (missing data)
      15 => None, // skip record status (always "added")
      _ => None,
    };

    if let Some(k) = k {
      attrs.insert(k.to_owned(), String::from_utf8_lossy(v).to_string());
    }
  }

  attrs
}
