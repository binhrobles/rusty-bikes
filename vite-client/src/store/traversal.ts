import { atom, computed } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import { Feature, Geometry } from 'geojson';

import Rainbow from 'rainbowvis.js';
const rainbow = new Rainbow();

import { StoredMarker } from './marker.ts';
import { Mode, TraversalDefaults, PaintOptions } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const { $marker, $latLng: $markerLatLng } = StoredMarker();
export const $depth = atom<number>(TraversalDefaults.depth);
export const $paint = atom<PaintOptions>(TraversalDefaults.paint);

// when mode switches away, clear marker and coords
$mode.listen((_, oldMode) => {
  if (oldMode === Mode.Traverse) {
    $marker.set(null);
  }
});

// tie the Traversal $marker to map $clicks when in Traverse $mode
$click.listen((event: LeafletMouseEvent | null) => {
  if ($mode.get() !== Mode.Traverse || !event) return;

  // create a new marker at the mouse click location
  const marker = new Marker(event.latlng, { draggable: true });

  // set the marker for map-related events
  $marker.set(marker);
});

export const $geoJsonRenderOptions =
  computed([$mode, $paint, $depth], (mode, paint, depth): L.GeoJSONOptions => {
    let style;

    switch (mode) {
      case Mode.RouteViz: {
          style = () => ({
            color: '#F26F75',
          });
        }
        break;
      case Mode.Traverse: {
          switch (paint) {
            case 'depth':
              rainbow.setNumberRange(1, depth);
              break;
            case 'length':
              rainbow.setNumberRange(1, 300);
              break;
            case 'distance_so_far':
              rainbow.setNumberRange(1, depth * 50);
              break;
            default:
          }

          style = (feature: Feature<Geometry, any> | undefined) => ({
            color: feature?.properties ?
              `#${rainbow.colourAt(feature.properties[paint])}` :
              '#000000', // if black is painted...we got issues!
          });
        }
        break;
      default:
    }

    return {
        style,
        // onEachFeature: makeFeatureClickable,
        bubblingMouseEvents: false,
    };
  }
);
