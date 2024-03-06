import { createSignal, Signal } from "solid-js";
import { createStore, SetStoreFunction, Store } from "solid-js/store";

function easeInOutCubic(t: number) {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

export function lerp(start: number, end: number, t: number) {
    return start * (1 - t) + end * t;
}

function easeInOutQuad(progress: number) {
    progress /= 0.5;
    if (progress < 1) return 0.5 * progress * progress;
    progress--;
    return -0.5 * (progress * (progress - 2) - 1);
}

export function cubic(start: number, end: number, t: number) {
    t = Math.max(0, Math.min(1, t));
    return start + (end - start) * easeInOutQuad(t);
}

export function randomSeed(): number {
    return self.crypto.getRandomValues(new Uint16Array(1))[0];
}

export type StoreAndSetter<T> = [Store<T>, SetStoreFunction<T>];
export function createLocalStore<T extends object>(
    name: string,
    init: T
): StoreAndSetter<T> {
    const localState = localStorage.getItem(name);
    const [state, setState] = createStore<T>(
        localState ? JSON.parse(localState) : init
    );

    // save score only periodically, saves a lot of updates
    setInterval(() => {
        localStorage.setItem(name, JSON.stringify(state));
    }, 500);

    return [state, setState];
}

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
