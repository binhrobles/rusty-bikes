/*
 * All config concerning the Routing mode
 */
import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import { Mode, HtmlElementId } from '../consts.ts';
import { StoredMarker } from './marker.ts';

import { $click } from './map.ts';
import { $mode } from './mode.ts';

export const { $marker: $startMarker, $latLng: $startMarkerLatLng } =
  StoredMarker();
export const { $marker: $endMarker, $latLng: $endMarkerLatLng } =
  StoredMarker();
export const $selectedInput = atom<
  HtmlElementId.StartInput | HtmlElementId.EndInput | null
>(null);

// when mode switches away, clear markers
$mode.listen((_, oldMode) => {
  if (oldMode === Mode.Route || oldMode === Mode.RouteViz) {
    [$startMarker, $endMarker].forEach(($marker) => {
      $marker.set(null);
    });
    $selectedInput.set(null);
  }
});

// tie the route $markers to map $clicks when in a Route $mode
$click.listen((event: LeafletMouseEvent | null) => {
  const mode = $mode.get();
  if ((mode !== Mode.Route && mode !== Mode.RouteViz) || !event) return;

  // create a new marker at the mouse click location
  const marker = new Marker(event.latlng, { draggable: true });

  // if one of the inputs were selected, change that one
  const selectedInput = $selectedInput.get();
  if (selectedInput) {
    const $m =
      selectedInput === HtmlElementId.StartInput ? $startMarker : $endMarker;

    $m.set(marker);

    // clear selected input bookmark
    $selectedInput.set(null);
    return;
  }

  // if no start marker, lay that down
  const start = $startMarker.get();
  const end = $endMarker.get();
  if (!start) {
    $startMarker.set(marker);
  } else if (!end) {
    // otherwise if no end marker, move the end marker
    $endMarker.set(marker);
  }

  // rely on dragging / clicking user input field to change marker location
});
