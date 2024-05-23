import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import { Mode, TraversalDefaults, PaintOptions } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $marker = atom<Marker | null>(null);
export const $depth = atom<number>(TraversalDefaults.depth);
export const $paint = atom<PaintOptions>(TraversalDefaults.paint);

// when mode switches away, clear marker
$mode.listen((_, oldMode) => {
  if (oldMode === Mode.Traverse) {
    $marker.get()?.remove();
    $marker.set(null);
  }
});

// tie the Traversal $marker to map $clicks when in Traverse $mode
$click.listen((event: LeafletMouseEvent | null) => {
  if ($mode.get() !== Mode.Traverse || !event) return;

  // remove the current marker if it exists
  $marker.get()?.remove();

  // create a new marker at the mouse click location
  $marker.set(new Marker(event.latlng, { draggable: true }));
});
