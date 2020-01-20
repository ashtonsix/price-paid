sudo apt update
sudo apt install unzip
sudo apt install python3-venv python3-pip
pip3 install awscli --upgrade --user
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

source .profile
aws configure

ssh -T git@github.com # make sure this says "ashtonsix"
git clone git@github.com:ashtonsix/price-paid.git
cd price-paid

mkdir data
cd data

mkdir tiles

mkdir shared
cd shared
wget http://prod.publicdata.landregistry.gov.uk.s3-website-eu-west-1.amazonaws.com/pp-complete.csv
wget https://www.freemaptools.com/download/full-postcodes/ukpostcodes.zip
unzip ukpostcodes.zip
cd ..

mkdir osm
cd osm
wget https://download.geofabrik.de/europe/ireland-and-northern-ireland-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/wales-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/scotland-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/bedfordshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/berkshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/bristol-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/buckinghamshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/cambridgeshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/cheshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/cornwall-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/cumbria-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/derbyshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/devon-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/dorset-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/durham-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/east-sussex-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/east-yorkshire-with-hull-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/essex-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/gloucestershire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/greater-london-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/greater-manchester-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/hampshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/herefordshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/hertfordshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/isle-of-wight-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/kent-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/lancashire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/leicestershire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/lincolnshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/merseyside-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/norfolk-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/north-yorkshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/northamptonshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/northumberland-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/nottinghamshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/oxfordshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/rutland-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/shropshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/somerset-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/south-yorkshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/staffordshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/suffolk-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/surrey-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/tyne-and-wear-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/warwickshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/west-midlands-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/west-sussex-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/west-yorkshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/wiltshire-latest.osm.bz2
wget https://download.geofabrik.de/europe/great-britain/england/worcestershire-latest.osm.bz2
ls | xargs bzip2 -d
cd ..

aws sync . s3://ashtonsix-price-paid
