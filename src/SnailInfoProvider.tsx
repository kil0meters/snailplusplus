
import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { ShopKey, SHOP_KEYS } from "./ShopProvider";
import { createLocalStore } from "./utils";

export type SnailInfo = {
    key: string,
    names: string[],
    solvedCounts: number[],
    createdAts: number[],
};

const SNAIL_INFO_DEFAULT: SnailInfo[] = SHOP_KEYS.map((key) => {
    return {
        key,
        names: [],
        solvedCounts: [],
        createdAts: []
    };
});

export const SnailInfoContext = createContext<[SnailInfo[], SetStoreFunction<SnailInfo[]>]>();

const SnailInfoProvider: Component<{ children: JSX.Element }> = (props) => {
    const [snailInfoItems, setSnailInfo] = createLocalStore<SnailInfo[]>("snail-info", SNAIL_INFO_DEFAULT);

    for (let info of SNAIL_INFO_DEFAULT) {
        let found = snailInfoItems.find(x => x.key == info.key)
        if (!found) {
            setSnailInfo([...snailInfoItems, info]);
        }
    }

    return (
        <SnailInfoContext.Provider value={[snailInfoItems, setSnailInfo]}>
            {props.children}
        </SnailInfoContext.Provider>
    );
}

export default SnailInfoProvider;
