import { atom } from 'nanostores';

// When true, re-routes silently with a toast; when false, shows confirmation prompt
export const $autoReroute = atom<boolean>(false);
