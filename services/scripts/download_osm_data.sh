curl --request POST \
  --url https://overpass-api.de/api/interpreter \
  --header 'Content-Type: application/x-www-form-urlencoded' \
  --data '
      [out:json][timeout:600];
      (
        area["name"="Manhattan"]["boundary"="administrative"];
        area["name"="Brooklyn"]["boundary"="administrative"];
        area["name"="Queens"]["boundary"="administrative"];
        area["name"="The Bronx"]["boundary"="administrative"];
      )->.boroughs;
      way(area.boroughs)
        ["highway"]
        [!"footway"]
        ["highway"!="footway"]
        ["highway"!="motorway"]
        ["highway"!="motorway_link"]
        ["highway"!="trunk"]
        ["highway"!="trunk_link"]
        ["highway"!="bridleway"]
        ["highway"!="raceway"]
        ["highway"!="services"]
        ["highway"!="rest_area"]
        ["highway"!="construction"]
        ["highway"!="steps"]
        ["highway"!="street_lamp"]
        ["highway"!="elevator"]
        ["highway"!="bus_stop"]
        ["highway"!="platform"]
        ["highway"!="proposed"]
        ["bicycle"!="no"]
        ;
      out geom;
    ' > out.geom.json
