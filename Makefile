BUILD=.aws-sam/build

## ------------ Running Locally ------------ ##
# for running this project locally at localhost:9000
service-watch: db.db3
	cd services && cargo lambda watch

service-test: db.db3
	cd services && DB_PATH=../db.db3 cargo test

service-bench: db.db3
	cd services && DB_PATH=../db.db3 cargo test --bench basic-benchmarking -- --show-output | tee benches/current.out


## ------------ Deploying ------------ ##
# deploys services to AWS
deploy: service-build service-deploy

# SAM toolkit helpers for deploying to AWS
service-build:
	sam validate
	sam build

service-deploy:
	sam deploy --no-confirm-changeset

service-clean:
	rm -rf .aws-sam/build/

## ------------ OSM / DB ------------ ##
# download you some OSM data
osm-download out.geom.json:
	./services/scripts/download_osm_data.sh

# build you a SQLite DB from the provided geojson
db-build db.db3: out.geom.json
ifneq (,$(wildcard db.db3))
	echo "first moving db to db.db3.bak..."
	mv db.db3 db.db3.bak
endif
	cd services && DB_PATH=../db.db3 cargo run --bin init-db
	cd services && DB_PATH=../db.db3 cargo run --bin populate-db ../out.geom.json

## ------------ DB Lambda Layer ------------ ##
# build a lambda layer artifact from the sqlite db
layer-build $(BUILD)/nyc-sqlite-db-layer.zip: db.db3
	mkdir -p $(BUILD)/lib/
	cp db.db3 $(BUILD)/lib/
	# cd into build directory to do the zipping (heckin relative pathing)
	cd $(BUILD) && zip -r nyc-sqlite-db-layer.zip lib

# uploads the lambda layer to my special lambda layer s3 bucket
layer-upload: $(BUILD)/nyc-sqlite-db-layer.zip
	aws s3 cp $(BUILD)/nyc-sqlite-db-layer.zip s3://rusty-bikes-osm-data/
	echo "Make sure to update template.yml with a new layer name"
