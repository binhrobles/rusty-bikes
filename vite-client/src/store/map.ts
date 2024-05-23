/*
 * Just forwards map clicks into a pub/sub-able atom
 *
 */
import { atom } from 'nanostores';
import L from 'leaflet';

export const $click = atom<L.LeafletMouseEvent | null>(null);
