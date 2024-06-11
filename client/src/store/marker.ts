/**
 * Abstraction for creating / working w/ draggable Markers and MarkerLatLngs
 * Adds ability to react to changes in _just_ the latlng of the marker
 */
import { atom, WritableAtom } from 'nanostores';
import { Marker, LatLng } from 'leaflet';

type StoredMarker = {
  $marker: WritableAtom<Marker | null>;
  $latLng: WritableAtom<LatLng | null>;
};

export const StoredMarker = (): StoredMarker => {
  const $marker = atom<Marker | null>(null);
  const $latLng = atom<LatLng | null>(null);

  // update the latLon and attach a handler to update on the drag event
  $marker.listen((marker) => {
    $latLng.set(marker?.getLatLng() || null);
    marker?.on('moveend', async () => {
      console.log(`moveend: ${marker?.getLatLng()}`);
      $latLng.set(marker?.getLatLng());
    });

  });

  return {
    $marker,
    $latLng,
  };
};
