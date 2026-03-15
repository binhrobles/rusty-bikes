import { atom } from 'nanostores';

export type AppView = 'planning' | 'navigating';

export const $appView = atom<AppView>('planning');
