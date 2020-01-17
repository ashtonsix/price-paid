cd join
cargo run -- \
  --pp data/shared/pp-complete.csv \
  --osm $(ls -rtd data/osm/*) \
  --postcode data/shared/ukpostcodes.csv \
  --output data/jsonl
