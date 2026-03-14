BUILD=.aws-sam/build

.PHONY: help
help:
	@echo "Rusty Bikes Makefile Commands"
	@echo ""
	@awk '/^## / {if (NR>1) print ""; section=$$2 " " $$3 " " $$4; gsub(/--+/, "", section); gsub(/^ +| +$$/, "", section); print section; next} /^[a-z-]+.*:/ && !/^\./ {match($$0, /^([a-z-]+)/); print "  make " substr($$0, RSTART, RLENGTH)}' Makefile
	@echo ""

## ------------ Running Locally ------------ ##
# for running this project locally at localhost:9000
service-watch: db.db3
ifdef release
	cd services && cargo lambda watch --release
else
	cd services && cargo lambda watch
endif

client-watch:
ifdef release
	cd client && yarn build && yarn preview
else
	cd client && yarn dev
endif

client-mobile-watch:
ifdef release
	cd client-mobile && yarn build && yarn preview
else
	cd client-mobile && yarn dev
endif

service-test: db.db3
	cd services && DB_PATH=../db.db3 cargo test

service-bench: db.db3
	cd services && DB_PATH=../db.db3 cargo test --bench basic-benchmarking -- --show-output | tee benches/current.out

service-flamegraph: db.db3
	cd services && CARGO_PROFILE_RELEASE_DEBUG=true DB_PATH=../db.db3 cargo flamegraph --root='--preserve-env' --bin=basic-benchmarking

## ------------ Deploying ------------ ##
# deploys services to AWS
service-deploy: sam-build sam-deploy

# SAM toolkit helpers for deploying to AWS
sam-build:
	sam validate
	sam build

sam-deploy:
	sam deploy --no-confirm-changeset

sam-clean:
	rm -rf .aws-sam/build/

## ------------ OSM and DB ------------ ##
# download you some OSM data
osm-download out.geom.json:
	./services/scripts/download_osm_data.sh

# download USGS 3DEP elevation rasters for NYC area
# n41w074 covers most of NYC (74°W-73°W), n41w075 covers western edge (75°W-74°W)
elevation-download:
	curl -o elevation_east.tif "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/13/TIFF/current/n41w074/USGS_13_n41w074.tif"
	curl -o elevation_west.tif "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/13/TIFF/current/n41w075/USGS_13_n41w075.tif"
	gdal_merge.py -o elevation.tif elevation_east.tif elevation_west.tif
	rm elevation_east.tif elevation_west.tif

# build you a SQLite DB from the provided geojson
# if elevation.tif is present, elevation data will be computed per-segment
db-build db.db3: out.geom.json
ifneq (,$(wildcard db.db3))
	echo "first moving db to db.db3.bak..."
	mv db.db3 db.db3.bak
endif
	cd services && DB_PATH=../db.db3 cargo run --bin init-db
ifneq (,$(wildcard elevation.tif))
	cd services && DB_PATH=../db.db3 ELEVATION_PATH=../elevation.tif cargo run --features elevation --bin populate-db ../out.geom.json
else
	cd services && DB_PATH=../db.db3 cargo run --bin populate-db ../out.geom.json
endif

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
