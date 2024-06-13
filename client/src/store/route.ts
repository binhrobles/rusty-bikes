/*
 * All config concerning the Routing
 */
import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';
import { HtmlElementId } from '../consts.ts';
import { StoredMarker } from './marker.ts';

import { $click } from './map.ts';

export const $withTraversal = atom<boolean>(false);
export const { $marker: $startMarker, $latLng: $startMarkerLatLng } =
  StoredMarker();
export const { $marker: $endMarker, $latLng: $endMarkerLatLng } =
  StoredMarker();
export const $selectedInput = atom<
  HtmlElementId.StartInput | HtmlElementId.EndInput | null
>(null);

// tie the route $markers to map $clicks
$click.listen((event: LeafletMouseEvent | null) => {
  if (!event) return;

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
