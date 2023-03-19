import { Component, createContext, JSX } from "solid-js";
import { createStore, SetStoreFunction, produce } from "solid-js/store";
import { ShopKey, SHOP_KEYS } from "./ShopProvider";

export type MazeAverage = {
    key: ShopKey;
    count: number;
    seconds: number;
};

const MAZE_AVERAGE_DEFAULT: MazeAverage[] = SHOP_KEYS.map((key) => {
    return { key, count: 0, seconds: 1 };
});

const AVERAGES_TEMPORARY =
    Object.fromEntries(SHOP_KEYS.map((key) =>
        [key, 0]
    )) as { [key in ShopKey]: number };

const setAveragesWrapper = (
    target: ShopKey,
    update: (prev: number) => number) => {
    AVERAGES_TEMPORARY[target] = update(AVERAGES_TEMPORARY[target]);
}

export const AverageContext = createContext<[MazeAverage[], typeof setAveragesWrapper]>();
const AverageProvider: Component<{ children: JSX.Element }> = (props) => {
    const [averages, setAverages] = createStore<MazeAverage[]>(structuredClone(MAZE_AVERAGE_DEFAULT));

    setInterval(() => {
        setAverages((prev) => {
            let newAverages = structuredClone(prev);

            for (let i = 0; i < prev.length; i++) {
                newAverages[i].count = AVERAGES_TEMPORARY[newAverages[i].key];
                newAverages[i].seconds = 30;
            }

            return newAverages;
        });

        for (let i = 0; i < SHOP_KEYS.length; i++) {
            AVERAGES_TEMPORARY[SHOP_KEYS[i]] = 0;
        }
    }, 30000);

    return (
        <AverageContext.Provider value={[averages, setAveragesWrapper]}>
            {props.children}
        </AverageContext.Provider>
    );
}
export default AverageProvider;
