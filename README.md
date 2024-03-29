### Options Request
Should return

```
Access-Control-Allow-Headers:
Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Authorization, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers
Access-Control-Allow-Methods:
OPTIONS,GET,POST
Access-Control-Allow-Origin:
*
Access-Control-Max-Age:
1728000
Date:
Fri, 29 Mar 2024 20:18:02 GMT
Server:
nginx/1.23.1
```

### POST Request
```
{
    "type": "FeatureCollection",
    "metadata": {
        "attribution": "openrouteservice.org | OpenStreetMap contributors",
        "service": "routing",
        "timestamp": 1711743482386,
        "query": {
            "coordinates": [
                [
                    0.048322677612304694,
                    51.47993965082005
                ],
                [
                    0.04858016967773438,
                    51.48303989917523
                ]
            ],
            "profile": "cycling-regular",
            "preference": "recommended",
            "format": "geojson",
            "units": "km",
            "language": "en",
            "instructions_format": "html",
            "elevation": true,
            "extra_info": [
                "surface",
                "steepness",
                "waytype"
            ]
        },
        "engine": {
            "version": "7.1.1",
            "build_date": "2024-01-29T14:41:12Z",
            "graph_date": "2024-03-17T22:17:33Z"
        }
    },
    "features": [
        {
            "bbox": [
                0.047243,
                51.48,
                45.19,
                0.048582,
                51.48308,
                49.0
            ],
            "type": "Feature",
            "properties": {
                "ascent": 0.0,
                "descent": 3.8,
                "transfers": 0,
                "fare": 0,
                "segments": [
                    {
                        "distance": 0.391,
                        "duration": 78.2,
                        "steps": [
                            {
                                "distance": 0.242,
                                "duration": 48.5,
                                "type": 11,
                                "instruction": "Head north on <b>Cemetery Lane</b>",
                                "name": "Cemetery Lane",
                                "way_points": [
                                    0,
                                    13
                                ]
                            },
                            {
                                "distance": 0.149,
                                "duration": 29.7,
                                "type": 1,
                                "instruction": "Turn right onto <b>Park Drive</b>",
                                "name": "Park Drive",
                                "way_points": [
                                    13,
                                    14
                                ]
                            },
                            {
                                "distance": 0.0,
                                "duration": 0.0,
                                "type": 10,
                                "instruction": "Arrive at Park Drive, on the right",
                                "name": "-",
                                "way_points": [
                                    14,
                                    14
                                ]
                            }
                        ],
                        "descent": 3.8125
                    }
                ],
                "extras": {
                    "surface": {
                        "values": [
                            [
                                0,
                                13,
                                3
                            ],
                            [
                                13,
                                14,
                                1
                            ]
                        ],
                        "summary": [
                            {
                                "value": 3.0,
                                "distance": 0.0,
                                "amount": 61.99
                            },
                            {
                                "value": 1.0,
                                "distance": 0.0,
                                "amount": 38.01
                            }
                        ]
                    },
                    "waytypes": {
                        "values": [
                            [
                                0,
                                13,
                                2
                            ],
                            [
                                13,
                                14,
                                3
                            ]
                        ],
                        "summary": [
                            {
                                "value": 2.0,
                                "distance": 0.0,
                                "amount": 61.99
                            },
                            {
                                "value": 3.0,
                                "distance": 0.0,
                                "amount": 38.01
                            }
                        ]
                    },
                    "steepness": {
                        "values": [
                            [
                                0,
                                14,
                                0
                            ]
                        ],
                        "summary": [
                            {
                                "value": 0.0,
                                "distance": 0.0,
                                "amount": 100.0
                            }
                        ]
                    }
                },
                "way_points": [
                    0,
                    14
                ],
                "summary": {
                    "distance": 0.391,
                    "duration": 78.2
                }
            },
            "geometry": {
                "coordinates": [
                    [
                        0.048582,
                        51.48,
                        49.0
                    ],
                    [
                        0.048573,
                        51.480015,
                        49.0
                    ],
                    [
                        0.04829,
                        51.480305,
                        49.0
                    ],
                    [
                        0.048109,
                        51.480598,
                        49.0
                    ],
                    [
                        0.048038,
                        51.480734,
                        49.0
                    ],
                    [
                        0.047953,
                        51.480898,
                        49.0
                    ],
                    [
                        0.047901,
                        51.480997,
                        49.0
                    ],
                    [
                        0.047823,
                        51.481138,
                        48.8
                    ],
                    [
                        0.047658,
                        51.481503,
                        47.9
                    ],
                    [
                        0.047644,
                        51.481529,
                        47.7
                    ],
                    [
                        0.04754,
                        51.481745,
                        47.4
                    ],
                    [
                        0.04747,
                        51.481828,
                        47.3
                    ],
                    [
                        0.047359,
                        51.481923,
                        47.1
                    ],
                    [
                        0.047243,
                        51.481991,
                        46.9
                    ],
                    [
                        0.04849,
                        51.48308,
                        45.2
                    ]
                ],
                "type": "LineString"
            }
        }
    ],
    "bbox": [
        0.047243,
        51.48,
        45.19,
        0.048582,
        51.48308,
        49.0
    ]
}
```
