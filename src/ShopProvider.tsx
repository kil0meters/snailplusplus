import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();

export const SHOP_KEYS = ["random-walk", "random-teleport", "hold-left", "tremaux", "clone", "time-travel", "learning", "rpg", "meta", "inverted"] as const;
export type ShopKey = typeof SHOP_KEYS[number];

export interface ShopItem {
    name: string;
    description: string;
    price: number;
    baseMultiplier: number;
    fragmentsPerSecond: number;
    latticeWidth: number;
};

export interface ShopListing {
    key: ShopKey;
    count: number;
};

const SHOP_LISTINGS_DEFAULT: ShopListing[] = SHOP_KEYS.map((key) => {
    return { key, count: 0 };
});

export const SHOP: { [key in ShopKey]: ShopItem } = {
    "random-walk": {
        "name": "Random Walk Snail",
        "description": "Randomly walks around until it happens to stumble its way to the end.",
        "price": 25,
        "baseMultiplier": 1,
        "latticeWidth": 8,
        "fragmentsPerSecond": 0.3,
    },
    "random-teleport": {
        "name": "Random Teleport Snail",
        "description": "Randomly teleports to another location.",
        "price": 100,
        "baseMultiplier": 1.5,
        "latticeWidth": 5,
        "fragmentsPerSecond": 1,
    },
    "learning": {
        "name": "Learning Snail",
        "description": "Learns how to solve the maze.",
        "price": 1_000,
        "baseMultiplier": 5,
        "latticeWidth": 3,
        "fragmentsPerSecond": 6,
    },
    "hold-left": {
        "name": "Left Handed Snail",
        "description": "Holds the left wall of the maze until it finds its way to the end. At least it's not unbounded!",
        "price": 50_000,
        "baseMultiplier": 5,
        "latticeWidth": 4,
        "fragmentsPerSecond": 25,
    },
    "inverted": {
        "name": "Right Handed Snail",
        "description": "Holds the right wall instead. I wonder where he got that idea from.",
        "price": 500_000,
        "baseMultiplier": 24,
        "latticeWidth": 4,
        "fragmentsPerSecond": 92,
    },
    "tremaux": {
        "name": "Segment Snail",
        "description": "Uses marks on the ground to block off segments of the maze which have been explored.",
        "price": 3_000_000,
        "baseMultiplier": 100,
        "latticeWidth": 3,
        "fragmentsPerSecond": 420,
    },
    "rpg": {
        "name": "RPG Snail",
        "description": "It's dangerous to go alone, go that way.",
        "price": 10_000_000,
        "baseMultiplier": 1_000,
        "latticeWidth": 3,
        "fragmentsPerSecond": 2800,
    },
    "time-travel": {
        "name": "Time Travel Snail",
        "description": "The segment snail has developed time travel. It travels back in time to when the maze was first conceived, then solves it using the method it previously invented. When it returns to the present it is able to use the markings to walk directly to the exit.",
        "price": 50_000_000,
        "baseMultiplier": 1_500,
        "latticeWidth": 3,
        "fragmentsPerSecond": 13_000,
    },
    "clone": {
        "name": "Cloning Snail",
        "description": "Can't turn but clones itself facing another direction when it reaches a junction.",
        "price": 800_000_000,
        "baseMultiplier": 4_000,
        "latticeWidth": 2,
        "fragmentsPerSecond": 80_000,
    },
    "meta": {
        "name": "Meta Snail",
        "description": "All the snails that came before.",
        "price": 10_000_000_000,
        "baseMultiplier": 14_000,
        "latticeWidth": 2,
        "fragmentsPerSecond": 500_000,
    }
};

const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
    const [shop, setShop] = createLocalStore<ShopListing[]>("shop", SHOP_LISTINGS_DEFAULT);

    // add new things to local shop if key is missing, only run at start
    for (let listing of SHOP_LISTINGS_DEFAULT) {
        if (!shop.find(x => x.key == listing.key)) {
            setShop([...shop, listing]);
            break;
        }
    }

    return (
        <ShopContext.Provider value={[shop, setShop]}>
            {props.children}
        </ShopContext.Provider>
    );
}

export default ShopProvider;
