/*
 * All config concerning the Traversal mode
 */
import { atom } from 'nanostores';
import { Marker, LeafletMouseEvent } from 'leaflet';

import { StoredMarker } from './marker.ts';
import { Mode, TraversalDefaults, PaintOptions, HtmlElementId } from '../consts.ts';

import { $click, $clickTime } from './map.ts';
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

  console.log(`marker created @ ${Date.now() - $clickTime.get()}`)
});

// bind the panel's bubbled up change events to the appropriate state changes
const bind = () => {
  document
    .getElementById(HtmlElementId.PanelParent)
    ?.addEventListener('change', (event: Event) => {
      const target = event.target as HTMLElement;

      switch (target.id) {
        // Traversal DOM event handlers
        case HtmlElementId.DepthRange:
          {
            const value = (target as HTMLInputElement).value;

            const depthValue = document.getElementById(
              HtmlElementId.DepthValue
            );
            if (!depthValue) throw "depthValue wasn't ready";

            depthValue.innerText = value;
            $depth.set(Number(value));
          }
          break;
        case HtmlElementId.PaintSelect:
          {
            const paint = (target as HTMLSelectElement).value as PaintOptions;
            $paint.set(paint);
          }
          break;

        default:
          console.error(`no onChange event handler for ${target.id}`);
      }
    });
};

export default {
  bind,
};
