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


const BASES = {
    6: "million",
    9: "billion",
    12: "trillion",
    15: "quadrillion",
    18: "quintillion",
};

// can't use Intl.NumberFormat because we can't control the rounding behavior.
// we want all prices in the shop to round up, but all prices the user owns to
// round down
export function formatNumber(num: bigint | number, roundUp: boolean): string {
    let value = Number(num);

    if (value < 1_000_000) {
        return value.toLocaleString('en', { maximumFractionDigits: 3 });
    } else if (value < 1e24) {
        let digits = 0;

        while (value >= 1_000) {
            digits += 3;

            value /= 1_000;
        }

        let rounded: number;

        if (roundUp) {
            rounded = Math.ceil(value * 1000) / 1000;
        } else {
            rounded = Math.floor(value * 1000) / 1000;
        }


        return `${rounded} ${BASES[digits]}`;

    } else {
        return value.toExponential(5)
    }
}
