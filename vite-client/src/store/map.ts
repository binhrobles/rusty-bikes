import { atom } from 'nanostores';
import { LeafletMouseEvent } from 'leaflet';

export const $click = atom<LeafletMouseEvent | null>(null);

export default {
  $click,
}
