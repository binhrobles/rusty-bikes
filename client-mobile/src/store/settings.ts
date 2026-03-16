import { atom } from 'nanostores';

export type AppView = 'planning' | 'navigating';

export const $appView = atom<AppView>('planning');

/** True when a search input is focused (keyboard visible). */
export const $searchFocused = atom(false);
