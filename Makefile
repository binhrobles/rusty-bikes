BUILD=.aws-sam/build

all: build deploy

$(BUILD)/nyc-sqlite-db-layer.zip: osm-data/nyc.db3
	mkdir -p $(BUILD)/lib/
	cp osm-data/nyc.db3 $(BUILD)/lib/
	# can't wrap my head around relative pathing showing up in my zips
	cd $(BUILD) && zip -r nyc-sqlite-db-layer.zip lib

upload-layer: $(BUILD)/nyc-sqlite-db-layer.zip
	aws s3 cp $(BUILD)/nyc-sqlite-db-layer.zip s3://rusty-bikes-osm-data/
	echo "Make sure to update template.yml with a new layer name"

build:
	sam build

deploy:
	sam deploy --no-confirm-changeset

clean:
	rm -rf .aws-sam/build/
