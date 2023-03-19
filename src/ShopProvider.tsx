import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const SHOP_KEYS = [
    "random-walk",
    "random-teleport",
    "learning",
    "hold-left",
    "inverted",
    "tremaux",
    "rpg",
    "time-travel",
    "clone",
    "meta",
    "demolitionist",
    "flying",
    "telepathic",
    "automaton"
] as const;
export type ShopKey = typeof SHOP_KEYS[number];

export interface ShopItem {
    name: string;
    description: string;
    price: number;
    baseMultiplier: bigint;
    latticeWidth: number;
    bgcolor: string;
};

export interface ShopListing {
    key: ShopKey;
    count: number;
    displayWidth: number;
    collapsed: boolean;
};

export const SHOP: { [key in ShopKey]: ShopItem } = {
    "random-walk": {
        name: "Random Walk Snail",
        description: "Randomly walks around until it happens to stumble its way to the end.",
        price: 25,
        baseMultiplier: 25n,
        latticeWidth: 4,
        bgcolor: "#068fef",
    },
    "random-teleport": {
        name: "Random Teleport Snail",
        description: "Randomly teleports to another location.",
        price: 100,
        baseMultiplier: 74n,
        latticeWidth: 4,
        bgcolor: "#068fef",
    },
    "learning": {
        name: "Learning Snail",
        description: "Learns how to solve the maze.",
        price: 1_000,
        baseMultiplier: 5n * 81n,
        latticeWidth: 3,
        bgcolor: "#068fef",
    },
    "hold-left": {
        name: "Left Handed Snail",
        description: "Holds the left wall of the maze until it finds its way to the end. At least it's not unbounded!",
        price: 12_000,
        baseMultiplier: 5n * 81n,
        latticeWidth: 3,
        bgcolor: "#068fef",
    },
    "inverted": {
        name: "Right Handed Snail",
        description: "Holds the right wall instead. I wonder where he got that idea from.",
        price: 200_000,
        baseMultiplier: 4_000n,
        latticeWidth: 3,
        bgcolor: "#f97010",
    },
    "tremaux": {
        name: "Segment Snail",
        description: "Uses marks on the ground to block off segments of the maze which have been explored.",
        price: 1_800_000,
        baseMultiplier: 25_000n,
        latticeWidth: 3,
        bgcolor: "#068fef",
    },
    "rpg": {
        name: "RPG Snail",
        description: "It's dangerous to go alone, go that way.",
        price: 10_000_000,
        baseMultiplier: 1_000n * 121n,
        latticeWidth: 3,
        bgcolor: "#068fef",
    },
    "time-travel": {
        name: "Time Travel Snail",
        description: "The segment snail has developed time travel. It travels back in time to when the maze was first conceived, then solves it using the method it previously invented. When it returns to the present it is able to use the markings to walk directly to the exit.",
        price: 70_000_000,
        baseMultiplier: 1_500n * 169n,
        latticeWidth: 3,
        bgcolor: "#068fef",
    },
    "clone": {
        name: "Cloning Snail",
        description: "Can't turn but clones itself facing another direction when it reaches a junction.",
        price: 800_000_000,
        baseMultiplier: 1_600_000n,
        latticeWidth: 2,
        bgcolor: "#068fef",
    },
    "meta": {
        name: "Meta Snail",
        description: "All the snails that came before.",
        price: 6_000_000_000,
        baseMultiplier: 686_000n,
        latticeWidth: 2,
        bgcolor: "#068fef",
    },
    "demolitionist": {
        name: "Demolitionist Snail",
        description: "Destroys walls to make its way through the maze faster.",
        price: 32_000_000_000,
        baseMultiplier: 38_000_000n,
        latticeWidth: 3,
        bgcolor: "#550000",
    },
    "flying": {
        name: "Swarm Snail",
        description: "A flock of Swarm Snails use their powers of flight to complete mazes.",
        price: 200_000_000_000,
        baseMultiplier: 4_000_000n,
        latticeWidth: 3,
        bgcolor: "#550000",
    },
    "telepathic": {
        name: "Telepathic Snail",
        description: "Uses its telepathic tiles to rearrange the maze for faster solving.",
        price: 1_500_000_000_000,
        baseMultiplier: 360_000_000n,
        latticeWidth: 3,
        bgcolor: "#550000",
    },
    "automaton": {
        name: "Automaton Snail",
        description: "A hive mind that is able replicate itself according to the rules of a cellular automaton. Each snail which is created is counted as solving the maze.",
        price: 20_00_000_000_000,
        baseMultiplier: 3_000_000n,
        latticeWidth: 2,
        bgcolor: "#550000",
    }
};

const SHOP_LISTINGS_DEFAULT: ShopListing[] = SHOP_KEYS.map((key) => {
    return { key, count: 0, displayWidth: SHOP[key].latticeWidth, collapsed: false };
});

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();
const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
    const [shop, setShop] = createLocalStore<ShopListing[]>("shop", SHOP_LISTINGS_DEFAULT);

    // add new things to local shop if key is missing, only run at start
    for (let listing of SHOP_LISTINGS_DEFAULT) {
        if (!shop.find(x => x.key == listing.key)) {
            setShop([...shop, listing]);
            break;
        }
    }

    setShop((newShop) => {
        newShop.sort((a, b) => SHOP_KEYS.indexOf(a.key) - SHOP_KEYS.indexOf(b.key));

        // legacy migration code
        for (let i = 0; i < newShop.length; i++) {
            if (!("displayWidth" in newShop[i])) {
                newShop[i].displayWidth = SHOP[newShop[i].key].latticeWidth;
            }

            if (!("collapsed" in newShop[i])) {
                newShop[i].collapsed = false;
            }
        }

        return newShop;
    });

    return (
        <ShopContext.Provider value={[shop, setShop]}>
            {props.children}
        </ShopContext.Provider>
    );
}

export default ShopProvider;
