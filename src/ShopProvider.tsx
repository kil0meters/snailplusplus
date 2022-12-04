import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();

export type ShopKey = "random-walk" | "random-teleport" | "hold-left" | "tremaux" | "clone" | "time-travel";

export interface ShopItem {
  name: string;
  description: string;
  price: number;
  baseMultiplier: number;
  mazeSize: number;
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

export const shop: { [key in ShopKey]: ShopItem } = {
  "random-walk": {
    "name": "Random Walk",
    "description": "Randomly walks around until it happens to stumble its way to the end",
    "price": 25,
    "baseMultiplier": 1,
    "mazeSize": 5,
    "latticeWidth": 8
  },
  "random-teleport": {
    "name": "Random Teleport",
    "description": "Randomly teleports to another location",
    "price": 100,
    "baseMultiplier": 1.5,
    "mazeSize": 7,
    "latticeWidth": 5
  },
  "hold-left": {
    "name": "Hold Left Wall",
    "description": "At least it's not unbounded!",
    "price": 1000,
    "baseMultiplier": 1,
    "mazeSize": 9,
    "latticeWidth": 4
  },
  "tremaux": {
    "name": "Segment Snail",
    "description": "Uses marks on the ground to block off segments of the maze which have been explored.",
    "price": 5000,
    "baseMultiplier": 5,
    "mazeSize": 11,
    "latticeWidth": 3
  },
  "time-travel": {
    "name": "Time Travel Snail",
    "description": "The segment snail has developed time travel. It travels back in time to when the maze was first conceived, then solves it using the method it previously invented. When it returns to the present it is able to use the markings to walk directly to the exit.",
    "price": 20000,
    "baseMultiplier": 9,
    "mazeSize": 13,
    "latticeWidth": 3
  },
  "clone": {
    "name": "Cloning Snail",
    "description": "Can't turn but clones itself facing another direction when it reaches a junction.",
    "price": 50000,
    "baseMultiplier": 20,
    "mazeSize": 20,
    "latticeWidth": 2
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
