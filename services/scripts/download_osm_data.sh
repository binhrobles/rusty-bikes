# Whole Manhattan / BK bbox
# way(40.588061,-74.046498,40.829387,-73.837004)
#
# Manhattan up to Central Park + BK down to Prospect
# way(40.647941,-74.028837,40.755695,-73.907988)
#
# Same dataset, but shaved
# way(40.647941,-74.028837,40.665695,-73.907988)
#
# SF Bay Area
# way(37.675316,-122.513358,37.964420,-122.061327)

curl --request POST \
  --url https://overpass-api.de/api/interpreter \
  --header 'Content-Type: application/x-www-form-urlencoded' \
  --data '
      [out:json][timeout:90];
      way(40.588061,-74.046498,40.829387,-73.837004)
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
