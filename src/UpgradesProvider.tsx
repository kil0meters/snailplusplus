import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { createLocalStore } from "./utils";

export const UpgradesContext = createContext<[Upgrade[], SetStoreFunction<Upgrade[]>]>();

export type Upgrade = {
  key: string;
  owned: boolean;
};

export type UpgradeListing = {
  name: string;
  description: string;
  price: number;
};

const upgradesDefault: Upgrade[] = [
  {
    key: "glasses",
    owned: false
  },
  {
    key: "banana-peel",
    owned: false
  }
];

export const upgrades: { [key: string]: UpgradeListing } = {
  "faster-snails": {
    "name": "Speed",
    "description": "Makes auto snails twice as fast",
    "price": 200,
  },
  "glasses": {
    "name": "Glasses",
    "description": "Gives random walk snails some glasses so they will no longer run into walls",
    "price": 500,
  },
  "banana-peel": {
    "name": "Banana Peel",
    "description": "Makes random walk snails stand on banana peels that ",
    "price": 500,
  }
}


const UpgradesProvider: Component<{ children: JSX.Element }> = (props) => {
  const [upgradeItems, setUpgrades] = createLocalStore<Upgrade[]>("upgrades", upgradesDefault);

  for (let [key, _] of Object.entries(upgrades)) {
    if (!upgradeItems.find(x => x.key == key)) {
      setUpgrades([...upgradeItems, { key, owned: false }]);
    }
  }

  return (
    <UpgradesContext.Provider value={[upgradeItems, setUpgrades]}>
      {props.children}
    </UpgradesContext.Provider>
  );
}

export default UpgradesProvider;
