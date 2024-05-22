import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import { Mode, TraversalInitialState, PaintOptions } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $marker = atom<Marker | null>(null);
export const $depth = atom<number>(TraversalInitialState.depth);
export const $paint = atom<PaintOptions>(TraversalInitialState.paint);

// tie the Traversal $marker to map $clicks when in Traverse $mode
$click.listen((event: LeafletMouseEvent | null) => {
  if ($mode.get() !== Mode.Traverse || !event) return;

  // remove the current marker if it exists
  $marker.get()?.remove();

});
