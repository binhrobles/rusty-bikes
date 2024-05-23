/**
 * Global `mode` of the app
 *
 */
import { atom } from 'nanostores';
import { Mode } from '../consts.ts';

/*
 * Checks if the `mode` queryParam has been set to a valid Mode option,
 * otherwise returns Mode.Route
 */
const determineFirstMode = (): Mode => {
  const params = new URLSearchParams(document.location.search);
  const mode = params.get('mode');

  if (!(mode && Object.values<string>(Mode).includes(mode))) return Mode.Route;

  return mode as Mode;
};

export const $mode = atom<Mode>(determineFirstMode());
