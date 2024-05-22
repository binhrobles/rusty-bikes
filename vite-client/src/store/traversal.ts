import { atom } from 'nanostores';
import { marker as Marker, Marker as MarkerType, LeafletMouseEvent } from 'leaflet';
import { Mode, TraversalDefaults, PaintOptions } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $marker = atom<MarkerType | null>(null);
export const $depth = atom<number>(TraversalDefaults.depth);
export const $paint = atom<PaintOptions>(TraversalDefaults.paint);

// tie the Traversal $marker to map $clicks when in Traverse $mode
$click.listen((event: LeafletMouseEvent | null) => {
  if ($mode.get() !== Mode.Traverse || !event) return;

  // remove the current marker if it exists
  $marker.get()?.remove();

  // create a new marker at the mouse click location
  $marker.set(Marker(event.latlng, { draggable: true }));
});
