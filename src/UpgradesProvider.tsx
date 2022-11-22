import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const UpgradesContext = createContext<[Upgrade[], SetStoreFunction<Upgrade[]>]>();

export type Upgrade = {
  key: string;
  name: string;
  description: string;
  price: number;
  owned: boolean;
};

const upgradesDefault: Upgrade[] = [
  {
    "key": "glasses",
    "name": "Glasses",
    "description": "Gives random walk snails some glasses so they will no longer run into walls",
    "price": 500,
    owned: false
  }
];


const UpgradesProvider: Component<{ children: JSX.Element }> = (props) => {
  const [upgrades, setUpgrades] = createLocalStore<Upgrade[]>("upgrades", upgradesDefault);

  return (
    <UpgradesContext.Provider value={[upgrades, setUpgrades]}>
      {props.children}
    </UpgradesContext.Provider>
  );
}

export default UpgradesProvider;
