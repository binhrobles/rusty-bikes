# Whole Manhattan / BK bbox
# way(40.647941,-74.028837,40.755695,-73.907988)
#
# alternatively, for a smaller set
# way(40.647941,-74.028837,40.665695,-73.907988)

curl --request POST \
  --url https://overpass-api.de/api/interpreter \
  --header 'Content-Type: application/x-www-form-urlencoded' \
  --data '
      [out:json][timeout:90];
      way(40.647941,-74.028837,40.755695,-73.907988)
        ["highway"]
        [!"footway"]
        ["highway"!="footway"]
        ["highway"!="motorway"]
        ["highway"!="motorway_link"]
        ["highway"!="trunk"]
        ["highway"!="trunk_link"]
        ["highway"!="construction"]
        ["highway"!="steps"]
        ["highway"!="street_lamp"]
        ["highway"!="elevator"]
        ["highway"!="bus_stop"]
        ["highway"!="platform"]
        ["bicycle"!="no"]
        ;
      out geom;
    ' > out.geom.json
