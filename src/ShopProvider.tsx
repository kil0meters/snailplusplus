import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();

export type ShopKey = "random-walk" | "random-teleport" | "hold-left" | "tremaux" | "clone" | "time-travel" | "learning";

export interface ShopItem {
  name: string;
  description: string;
  price: number;
  baseMultiplier: number;
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
    key: "tremaux",
    count: 0
  },
  {
    key: "time-travel",
    count: 0
  },
  {
    key: "clone",
    count: 0
  }
];

export const SHOP: { [key in ShopKey]: ShopItem } = {
  "random-walk": {
    "name": "Random Walk Snail",
    "description": "Randomly walks around until it happens to stumble its way to the end",
    "price": 25,
    "baseMultiplier": 1,
    "latticeWidth": 8,
  },
  "random-teleport": {
    "name": "Random Teleport Snail",
    "description": "Randomly teleports to another location",
    "price": 100,
    "baseMultiplier": 1.5,
    "latticeWidth": 5,
  },
  "learning": {
    "name": "Learning Snail",
    "description": "Learns how to solve the maze",
    "price": 40_000,
    "baseMultiplier": 5,
    "latticeWidth": 3,
  },
  "hold-left": {
    "name": "Left Handed Snail",
    "description": "Holds the left wall of the maze until it finds its way to the end. At least it's not unbounded!",
    "price": 1_000,
    "baseMultiplier": 5,
    "latticeWidth": 4,
  },
  "tremaux": {
    "name": "Segment Snail",
    "description": "Uses marks on the ground to block off segments of the maze which have been explored.",
    "price": 200_000,
    "baseMultiplier": 24,
    "latticeWidth": 3,
  },
  "time-travel": {
    "name": "Time Travel Snail",
    "description": "The segment snail has developed time travel. It travels back in time to when the maze was first conceived, then solves it using the method it previously invented. When it returns to the present it is able to use the markings to walk directly to the exit.",
    "price": 1_000_000,
    "baseMultiplier": 50,
    "latticeWidth": 3,
  },
  "clone": {
    "name": "Cloning Snail",
    "description": "Can't turn but clones itself facing another direction when it reaches a junction.",
    "price": 10_000_000,
    "baseMultiplier": 100,
    "latticeWidth": 2,
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
