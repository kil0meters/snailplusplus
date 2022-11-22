import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const ShopContext = createContext<[ShopItem[], SetStoreFunction<ShopItem[]>]>();

type ShopKey = "random-walk" | "hold-left" | "tremaux";

export type ShopItem = {
  key: ShopKey;
  name: string;
  description: string;
  price: number;
  count: number;
}


export const shopItems: ShopItem[] = [
  {
    "key": "random-walk",
    "name": "Random Walk",
    "description": "Humble beginnings",
    "price": 100,
    "count": 0
  },
  {
    "key": "hold-left",
    "name": "Hold Left Wall",
    "description": "At least it's not unbounded!",
    "price": 1000,
    "count": 0
  },
  {
    "key": "tremaux",
    "name": "Trémaux's algorithm",
    "description": "It's french",
    "price": 2000,
    "count": 0
  }
];

const ShopProvider: Component<{ children: JSX.Element }> = (props) => {
  const [shop, setShop] = createLocalStore<ShopItem[]>("shop", shopItems);

  // add new things to local shop if key is missing, only run at start
  for (let item of shopItems) {
    setShop(
      x => x.key === item.key && x.price !== item.price,
      "price",
      () => item.price
    );

    setShop(
      x => x.key === item.key && x.description !== item.description,
      "description",
      () => item.description
    );

    if (!shop.find(x => x.key == item.key)) {
      setShop([...shop, item]);
    }

    // sort by price
    setShop([...shop].sort((a, b) => a.price - b.price));
    // setShop(shop);
  }

  return (
    <ShopContext.Provider value={[shop, setShop]}>
      {props.children}
    </ShopContext.Provider>
  );
}

export default ShopProvider;
