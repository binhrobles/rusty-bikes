/*
 * All config concerning the Traversal mode
 */
import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';

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
