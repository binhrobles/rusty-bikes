BUILD=.aws-sam/build

all: build $(BUILD)/nyc-sqlite-db-layer.zip deploy

$(BUILD)/nyc-sqlite-db-layer.zip: osm-data/nyc.db3
	mkdir -p $(BUILD)/lib/
	cp osm-data/nyc.db3 $(BUILD)/lib/
	# can't wrap my head around relative pathing showing up in my zips
	cd $(BUILD) && zip -r nyc-sqlite-db-layer.zip lib

build:
	sam build

deploy:
	sam deploy

clean:
	rm -rf .aws-sam/build/
