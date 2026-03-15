import { atom } from 'nanostores';

export type AppView = 'planning' | 'navigating';

export const $appView = atom<AppView>('planning');
export const $settingsOpen = atom<boolean>(false);
