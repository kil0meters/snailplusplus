import { createSignal, Signal } from "solid-js";
import { createStore, StoreSetter } from "solid-js/store";
import { WasmLattice } from "../../snail-lattice/pkg/snail_lattice";
import { createLocalStore, StoreAndSetter } from "../util";

const RANDOM_WALK_SNAIL_UPGRADES = ["four-leaf-clover", "rabbits-foot", "horseshoe"] as const;
type RandomWalkSnailUpgrades = typeof RANDOM_WALK_SNAIL_UPGRADES[number];

export const SNAIL_NAMES = ["random-walk", "random-teleport"] as const;
export type SnailKey = typeof SNAIL_NAMES[number];
type SnailInfo = {
    name: string;
    size: number,
    basePrice: string,
    upgrades: readonly UpgradeKey[],
    lattice: WasmLattice,
    mazes: any[],
    minWidth: number,
    store: StoreAndSetter<{
        count: number,
        upgrades: any,

        x: number,
        y: number,
        width: number,
        height: number,
    }>
};

export const SNAILS = {
    "random-walk": {
        name: "Random Walk Snail",
        size: 5,
        basePrice: "25",
        upgrades: RANDOM_WALK_SNAIL_UPGRADES,
        lattice: undefined,
        mazes: [],
        minWidth: 4,
        store: createLocalStore("random-walk", {
            count: 0,
            upgrades: {},

            x: 0,
            y: 0,
            width: 5,
            height: 5,
        }),
    },
    "random-teleport": {
        name: "Random Teleport Snail",
        size: 7,
        basePrice: "100",
        upgrades: RANDOM_WALK_SNAIL_UPGRADES,
        lattice: undefined,
        mazes: [],
        minWidth: 3,
        store: createLocalStore("random-teleport", {
            count: 0,
            upgrades: {},

            x: 0,
            y: 0,
            width: 4,
            height: 4,
        }),
    }
} as { [K in SnailKey]: SnailInfo };

export const UPGRADES: {
    [K in RandomWalkSnailUpgrades]: {
        name: string,
        icon: string,
        description: string,
        price: string,
        showAfter: number
    }
} = {
    "four-leaf-clover": {
        name: "Four Leaf Clover",
        icon: "🍀",
        description: "Gives Random Walk Snails and additional 10% chance to go in the right direction.",
        price: "400",
        showAfter: 5
    },
    "rabbits-foot": {
        name: "Rabbit's Foot",
        icon: "🐇",
        description: "Gives Random Walk Snails an additional 20% chance to go in the right direction.",
        price: "4000",
        showAfter: 25
    },
    "horseshoe": {
        name: "Horseshoe",
        icon: "🧲",
        description: "Gives Random Walk Snails an additional 30% chance to go in the right direction.",
        price: "50000",
        showAfter: 50
    }
}

export type UpgradeKey = keyof typeof UPGRADES;
