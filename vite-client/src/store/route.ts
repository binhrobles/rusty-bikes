import { atom } from 'nanostores';
import { marker as Marker, Marker as MarkerType, LeafletMouseEvent } from 'leaflet';
import { Mode, HtmlElementId } from '../consts.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const $startMarker = atom<MarkerType | null>(null);
export const $endMarker = atom<MarkerType | null>(null);
export const $selectedInput =
  atom<HtmlElementId.StartInput | HtmlElementId.EndInput | null>(null);

// when mode switches away, clear markers
$mode.listen((_, oldMode) => {
  if (oldMode === Mode.Route || oldMode === Mode.RouteViz) {
    [$startMarker, $endMarker].forEach($marker => {
      $marker.get()?.remove();
      $marker.set(null);
    });
    $selectedInput.set(null);
  }
});

// tie the route $markers to map $clicks when in a Route $mode
$click.listen((event: LeafletMouseEvent | null) => {
  const mode = $mode.get();
  if (mode !== Mode.Route && mode !== Mode.RouteViz || !event) return;

  // create a new marker at the mouse click location
  const marker = Marker(event.latlng, { draggable: true });

  // if one of the inputs were selected, change that one
  const selectedInput = $selectedInput.get();
  if (selectedInput) {
    const $m = selectedInput === HtmlElementId.StartInput ? $startMarker : $endMarker;

    // first remove, then set, then clear selected input bookmark
    $m.get()?.remove();
    $m.set(marker);
    $selectedInput.set(null);
    return;
  }

  // if no start marker, lay that down
  const start = $startMarker.get();
  if (!start) {
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