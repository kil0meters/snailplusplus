import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { ShopKey } from "./ShopProvider";
import { createLocalStore } from "./utils";

export const UpgradesContext = createContext<[Upgrade[], SetStoreFunction<Upgrade[]>]>();

const UPGRADE_KEYS = [
    "four-leaf-clover",
    "rabbits-foot",
    "horseshoe",
    "fusion-reactor",
    "homing-beacon",
    "advanced-homing-beacon",
    "population-boom",
    "uranium",
    "radium"
] as const;
export type UpgradeKey = typeof UPGRADE_KEYS[number];

export type Upgrade = {
    key: UpgradeKey;
    owned: boolean;
};

export type UpgradeListing = {
    name: string;
    icon: string;
    description: string;
    price: number;
    order: number; // where in the progression the upgrade is
    mazeType: ShopKey;
    showAfter: number; // certain number of units of that type
};


const UPGRADES_DEFAULT: Upgrade[] = UPGRADE_KEYS.map((key) => { return { key, owned: false } });

export const UPGRADES: { [key: string]: UpgradeListing } = {
    "four-leaf-clover": {
        name: "Four Leaf Clover",
        icon: "🍀",
        description: "Gives Random Walk Snails an additional 10% chance to go the right direction.",
        price: 400,
        order: 0,
        mazeType: "random-walk",
        showAfter: 5,
    },
    "rabbits-foot": {
        name: "Rabbit's Foot",
        icon: "🐇",
        description: "Gives Random Walk Snails an additional 10% chance to go the right direction.",
        price: 4000,
        order: 1,
        mazeType: "random-walk",
        showAfter: 25,
    },
    "horseshoe": {
        name: "Rabbit's Foot",
        icon: "🧲",
        description: "Gives Random Walk Snails an additional 10% chance to go the right direction.",
        price: 10_000,
        order: 2,
        mazeType: "random-walk",
        showAfter: 50,
    },
    "fusion-reactor": {
        name: "Fusion Reactor",
        icon: "☄️",
        description: "Random Teleport Snail uses a fusion reactor to charge up its teleports 20% faster.",
        price: 2_000,
        order: 0,
        mazeType: "random-teleport",
        showAfter: 5,
    },
    "homing-beacon": {
        name: "Homing Beacon",
        icon: "🔉",
        description: "Random Teleport Snail uses a homing beacon to get more accurate teleports over time.",
        price: 20_000,
        order: 1,
        mazeType: "random-teleport",
        showAfter: 25,
    },
    "advanced-homing-beacon": {
        name: "Advanced Homing Beacon",
        description: "Random Teleport Snail upgrades its homing beacon to get even more accurate teleports.",
        icon: "🔊",
        price: 1_000_000,
        order: 2,
        mazeType: "random-teleport",
        showAfter: 50
    },
    "population-boom": {
        name: "Population Boom",
        description: "A recent population boom has lead to larger generations of Learning Snails.",
        icon: "👥",
        price: 10_000,
        order: 0,
        mazeType: "learning",
        showAfter: 5
    },
    "uranium": {
        name: "Uranium Mine",
        description: "A nearby uranium mine leads to Learning Snails having a higher mutation rate.",
        icon: "☢️",
        price: 500_000,
        order: 0,
        mazeType: "learning",
        showAfter: 25
    },
    "radium": {
        name: "Radium Mine",
        description: "A nearby radium mine leads to Learning Snails having a higher mutation rate.",
        icon: "⚛️",
        price: 500_000,
        order: 0,
        mazeType: "learning",
        showAfter: 25
    },
}


const UpgradesProvider: Component<{ children: JSX.Element }> = (props) => {
    const [upgradeItems, setUpgrades] = createLocalStore<Upgrade[]>("upgrades", UPGRADES_DEFAULT);

    for (let upgrade of UPGRADES_DEFAULT) {
        if (!upgradeItems.find(x => x.key == upgrade.key)) {
            setUpgrades([...upgradeItems, upgrade]);
        }
    }

    for (let upgrade of upgradeItems) {
        if (!UPGRADES_DEFAULT.find(x => x.key == upgrade.key)) {
            setUpgrades([...UPGRADES_DEFAULT]);
            break;
        }
    }

    return (
        <UpgradesContext.Provider value={[upgradeItems, setUpgrades]}>
            {props.children}
        </UpgradesContext.Provider>
    );
}

export default UpgradesProvider;
