import { atom } from 'nanostores';
import { marker as Marker, Marker as MarkerType, LeafletMouseEvent } from 'leaflet';
import { Mode } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $startMarker = atom<MarkerType | null>(null);
export const $endMarker = atom<MarkerType | null>(null);

// tie the route $markers to map $clicks when in a Route $mode
$click.listen((event: LeafletMouseEvent | null) => {
  const mode = $mode.get();
  if (![Mode.Route, Mode.RouteViz].includes(mode) || !event) return;

  const marker = Marker(event.latlng, { draggable: true });

  // if no start marker, lay that down
  const start = $startMarker.get();
  if (!start) {
    // create a new marker at the mouse click location
    $startMarker.set(marker);
    return;
  }

  // if no end marker, lay _that_ down
  const end = $endMarker.get();
  if (!end) {
    // create a new marker at the mouse click location
    $endMarker.set(marker);
    return;
  } else {
    // otherwise, move the end marker
    end.remove();
    $endMarker.set(marker);
  }
});
