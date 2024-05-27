/*
 * Just forwards map clicks into a pub/sub-able atom
 */
import { atom, computed } from 'nanostores';
import { LeafletMouseEvent } from 'leaflet';

export const $click = atom<LeafletMouseEvent | null>(null);
export const $clickTime = computed($click, (_) => Date.now());
