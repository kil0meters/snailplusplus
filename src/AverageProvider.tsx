import { Component, createContext, JSX } from "solid-js";
import { createStore, SetStoreFunction } from "solid-js/store";
import { ShopKey, SHOP_KEYS } from "./ShopProvider";

export type MazeAverage = {
    key: ShopKey;
    count: number;
    seconds: number;
};

const MAZE_AVERAGE_DEFAULT: MazeAverage[] = SHOP_KEYS.map((key) => {
    return { key, count: 0, seconds: 1 };
});

export const AverageContext = createContext<[MazeAverage[], SetStoreFunction<MazeAverage[]>]>();
const AverageProvider: Component<{ children: JSX.Element }> = (props) => {
    const [averages, setAverages] = createStore<MazeAverage[]>(MAZE_AVERAGE_DEFAULT);

    return (
        <AverageContext.Provider value={[averages, setAverages]}>
            {props.children}
        </AverageContext.Provider>
    );
}
export default AverageProvider;
