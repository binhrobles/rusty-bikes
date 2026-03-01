import { atom } from 'nanostores';

export const $isLoading = atom<boolean>(false);
export const $error = atom<string | null>(null);
