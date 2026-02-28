/*
 * All config concerning the Routing
 */
import { atom } from 'nanostores';
import Radar from 'radar-sdk-js';
import { HtmlElementId } from '../consts.ts';
import { StoredMarker } from './marker.ts';

export const $withTraversal = atom<boolean>(false);
export const { $marker: $startMarker, $latLng: $startMarkerLatLng } =
  StoredMarker();
export const { $marker: $endMarker, $latLng: $endMarkerLatLng } =
  StoredMarker();
export const $selectedInput = atom<
  HtmlElementId.StartInput | HtmlElementId.EndInput | null
>(null);

// Address text atoms — updated by autocomplete selection or reverse geocode
export const $startAddress = atom<string>('');
export const $endAddress = atom<string>('');

// Helper: reverse geocode a lat/lng and return formatted address
const reverseGeocode = async (lat: number, lng: number): Promise<string> => {
  try {
    const result = await Radar.reverseGeocode({ latitude: lat, longitude: lng });
    return result?.addresses?.[0]?.formattedAddress || `(${lng.toFixed(5)}, ${lat.toFixed(5)})`;
  } catch (e) {
    console.error('Reverse geocode failed:', e);
    return `(${lng.toFixed(5)}, ${lat.toFixed(5)})`;
  }
};

// reverse geocode when markers are dragged
$startMarkerLatLng.listen((latLng) => {
  if (latLng) {
    reverseGeocode(latLng.lat, latLng.lng).then((addr) => $startAddress.set(addr));
  }
});
$endMarkerLatLng.listen((latLng) => {
  if (latLng) {
    reverseGeocode(latLng.lat, latLng.lng).then((addr) => $endAddress.set(addr));
  }
});
