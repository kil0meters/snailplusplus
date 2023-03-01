import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();

export type ShopKey = "random-walk" | "random-teleport" | "hold-left" | "tremaux" | "clone" | "time-travel" | "learning" | "rpg" | "meta" | "inverted";

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

const shopListings: ShopListing[] = [
    {
        key: "random-walk",
        count: 0
    },
    {
        key: "random-teleport",
        count: 0
    },
    {
        key: "learning",
        count: 0,
    },
    {
        key: "hold-left",
        count: 0
    },
    {
        key: "inverted",
        count: 0
    },
    {
        key: "tremaux",
        count: 0
    },
    {
        key: "rpg",
        count: 0
    },
    {
        key: "time-travel",
        count: 0
    },
    {
        key: "clone",
        count: 0
    },
    {
        key: "meta",
        count: 0
    },
];


export const SHOP: { [key in ShopKey]: ShopItem } = {
    "random-walk": {
        "name": "Random Walk Snail",
        "description": "Randomly walks around until it happens to stumble its way to the end.",
        "price": 25,
        "baseMultiplier": 100,
        "latticeWidth": 8,
        "fragmentsPerSecond": 0.3,
    },
    "random-teleport": {
        "name": "Random Teleport Snail",
        "description": "Randomly teleports to another location.",
        "price": 100,
        "baseMultiplier": 150,
        "latticeWidth": 5,
        "fragmentsPerSecond": 1,
    },
    "learning": {
        "name": "Learning Snail",
        "description": "Learns how to solve the maze.",
        "price": 1_000,
        "baseMultiplier": 500,
        "latticeWidth": 3,
        "fragmentsPerSecond": 6,
    },
    "hold-left": {
        "name": "Left Handed Snail",
        "description": "Holds the left wall of the maze until it finds its way to the end. At least it's not unbounded!",
        "price": 50_000,
        "baseMultiplier": 500,
        "latticeWidth": 4,
        "fragmentsPerSecond": 25,
    },
    "inverted": {
        "name": "Right Handed Snail",
        "description": "Holds the right wall instead. I wonder where he got that idea from.",
        "price": 500_000,
        "baseMultiplier": 2400,
        "latticeWidth": 4,
        "fragmentsPerSecond": 92,
    },
    "tremaux": {
        "name": "Segment Snail",
        "description": "Uses marks on the ground to block off segments of the maze which have been explored.",
        "price": 3_000_000,
        "baseMultiplier": 10000,
        "latticeWidth": 3,
        "fragmentsPerSecond": 420,
    },
    "rpg": {
        "name": "RPG Snail",
        "description": "It's dangerous to go alone, go that way.",
        "price": 15_000_000,
        "baseMultiplier": 100000,
        "latticeWidth": 3,
        "fragmentsPerSecond": 2800,
    },
    "time-travel": {
        "name": "Time Travel Snail",
        "description": "The segment snail has developed time travel. It travels back in time to when the maze was first conceived, then solves it using the method it previously invented. When it returns to the present it is able to use the markings to walk directly to the exit.",
        "price": 100_000_000,
        "baseMultiplier": 150000,
        "latticeWidth": 3,
        "fragmentsPerSecond": 13_000,
    },
    "clone": {
        "name": "Cloning Snail",
        "description": "Can't turn but clones itself facing another direction when it reaches a junction.",
        "price": 1_000_000_000,
        "baseMultiplier": 400000,
        "latticeWidth": 2,
        "fragmentsPerSecond": 80_000,
    },
    "meta": {
        "name": "Meta Snail",
        "description": "All the snails that came before.",
        "price": 30_000_000_000,
        "baseMultiplier": 1_400_000,
        "latticeWidth": 2,
        "fragmentsPerSecond": 500_000,
    }
};

const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
    const [shop, setShop] = createLocalStore<ShopListing[]>("shop", shopListings);

    // add new things to local shop if key is missing, only run at start
    for (let listing of shopListings) {
        if (!shop.find(x => x.key == listing.key)) {
            setShop([...shopListings]);
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
