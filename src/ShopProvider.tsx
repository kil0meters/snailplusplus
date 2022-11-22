import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopItem[], SetStoreFunction<ShopItem[]>]>();

export type ShopItem = {
  key: string;
  name: string;
  description: string;
  price: number;
  count: number;
}

const shopItems: ShopItem[] = [
  {
    "key": "random-walk",
    "name": "Random Walk",
    "description": "Humble beginnings",
    "price": 100,
    "count": 0
  },
  {
    "key": "tremaux",
    "name": "Trémaux's algorithm",
    "description": "It's french",
    "price": 500,
    "count": 0
  }
];

const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
  const [shop, setShop] = createLocalStore<ShopItem[]>("shop", shopItems);

  return (
    <ShopContext.Provider value={[shop, setShop]}>
      {props.children}
    </ShopContext.Provider>
  );
}

export default ShopProvider;
