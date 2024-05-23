import { atom, computed } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import Rainbow from 'rainbowvis.js';
import { Feature, Geometry } from 'geojson';

import { Mode, TraversalDefaults, PaintOptions } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $marker = atom<Marker | null>(null);
export const $markerLatLng = atom<L.LatLng | null>(null);
export const $depth = atom<number>(TraversalDefaults.depth);
export const $paint = atom<PaintOptions>(TraversalDefaults.paint);

const rainbow = new Rainbow();

export const $geoJsonRenderOptions =
  computed([$paint, $depth], (paint, depth): L.GeoJSONOptions => {
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

    return {
        style: (feature: Feature<Geometry, any> | undefined) => ({
          color: feature?.properties ?
            `#${rainbow.colourAt(feature.properties[paint])}` :
            '#000000', // if black is painted...we got issues!
        }),
        // onEachFeature: makeFeatureClickable,
        bubblingMouseEvents: false,
    };
  }
);

// when mode switches away, clear marker and coords
$mode.listen((_, oldMode) => {
  if (oldMode === Mode.Traverse) {
    $marker.set(null);
    $markerLatLng.set(null);
  }
});

// tie the Traversal $marker to map $clicks when in Traverse $mode
$click.listen((event: LeafletMouseEvent | null) => {
  if ($mode.get() !== Mode.Traverse || !event) return;

  // create a new marker at the mouse click location
  const marker = new Marker(event.latlng, { draggable: true });

  // update the latLon and attach a handler to the draggable event
  $markerLatLng.set(marker.getLatLng());
  marker.on('move', async event => {
      $markerLatLng.set((event as L.LeafletMouseEvent).latlng);
  });

  // set the marker for map-related events
  $marker.set(marker);
});

