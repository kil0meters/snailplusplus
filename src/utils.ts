import { createEffect, createSignal, Signal } from "solid-js";
import { createStore, SetStoreFunction, Store } from "solid-js/store";

export function createStoredSignal<T>(
   key: string,
   defaultValue: T,
   storage = localStorage
): Signal<T> {

  const initialValue = storage.getItem(key)
    ? JSON.parse(storage.getItem(key)) as T
    : defaultValue;

  const [value, setValue] = createSignal<T>(initialValue);

  const setValueAndStore = ((arg: T) => {
    //@ts-ignore
    const v = setValue(arg);
    storage.setItem(key, JSON.stringify(v));
    return v;
  }) as typeof setValue;

  return [value, setValueAndStore];
}

export function generateMaze(width: number, height: number, then: (maze: Uint8Array) => void) {
  let mazeGenerationWorker = new Worker(new URL("./generateMaze.ts", import.meta.url));
  mazeGenerationWorker.postMessage({ width, height });
  mazeGenerationWorker.onmessage = (msg: MessageEvent<Uint8Array>) => {
    then(msg.data);
    mazeGenerationWorker.terminate();
  };
}

export function createLocalStore<T extends object>(
  name: string,
  init: T
): [Store<T>, SetStoreFunction<T>] {
  const localState = localStorage.getItem(name);
  const [state, setState] = createStore<T>(
    localState ? JSON.parse(localState) : init
  );
  createEffect(() => localStorage.setItem(name, JSON.stringify(state)));
  return [state, setState];
}
