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

export function randomSeed(): number {
    return self.crypto.getRandomValues(new Uint16Array(1))[0];
}

export function createLocalStore<T extends object>(
    name: string,
    init: T
): [Store<T>, SetStoreFunction<T>] {
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

export function bigint_min(a: bigint, b: bigint) {
    return a < b ? a : b;
}
