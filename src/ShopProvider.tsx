import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopListing[], SetStoreFunction<ShopListing[]>]>();

export type ShopKey = "random-walk" | "hold-left" | "tremaux" | "clone";

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
    key: "hold-left",
    count: 0
  },
  {
    key: "tremaux",
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
  "hold-left": {
    "name": "Hold Left Wall",
    "description": "At least it's not unbounded!",
    "price": 300,
    "baseMultiplier": 1,
    "mazeSize": 7,
    "latticeWidth": 6
  },
  "tremaux": {
    "name": "Trémaux's algorithm",
    "description": "Uses marks on the ground to block off segments of the maze which have been explored.",
    "price": 4000,
    "baseMultiplier": 5,
    "mazeSize": 9,
    "latticeWidth": 4
  },
  "clone": {
    "name": "Cloning Snail",
    "description": "Can't turn but clones itself facing another direction when it reaches a junction.",
    "price": 50000,
    "baseMultiplier": 2,
    "mazeSize": 20,
    "latticeWidth": 2
  }
};

const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
  const [shop, setShop] = createLocalStore<ShopListing[]>("shop", shopListings);

  // add new things to local shop if key is missing, only run at start
  for (let listing of shopListings) {
    if (!shop.find(x => x.key == listing.key)) {
      setShop([...shop, listing]);
    }
  }

  return (
    <ShopContext.Provider value={[shop, setShop]}>
      {props.children}
    </ShopContext.Provider>
  );
}

export default ShopProvider;
