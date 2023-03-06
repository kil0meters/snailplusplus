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
    "radium",
    "left-glove",
    "right-handed-snail",
    "right-glove",
    "left-handed-snail",
    "compass",
    "electromagnet",
    "breadcrumbs",
    "comradery",
    "sidequests",
    "recruitment",
    "forward-time-travel",
    "improved-time-relay",
    "time-warp",
    "self-improvement",
    "singularity"
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
    price: bigint;
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
        price: 400n,
        order: 0,
        mazeType: "random-walk",
        showAfter: 5,
    },
    "rabbits-foot": {
        name: "Rabbit's Foot",
        icon: "🐇",
        description: "Gives Random Walk Snails an additional 20% chance to go the right direction.",
        price: 4000n,
        order: 1,
        mazeType: "random-walk",
        showAfter: 25,
    },
    "horseshoe": {
        name: "Horseshoe",
        icon: "🧲",
        description: "Gives Random Walk Snails an additional 30% chance to go the right direction.",
        price: 50_000n,
        order: 2,
        mazeType: "random-walk",
        showAfter: 50,
    },
    "fusion-reactor": {
        name: "Fusion Reactor",
        icon: "☄️",
        description: "Random Teleport Snail uses a fusion reactor to charge up its teleports 20% faster.",
        price: 2_000n,
        order: 0,
        mazeType: "random-teleport",
        showAfter: 5,
    },
    "homing-beacon": {
        name: "Homing Beacon",
        icon: "🔉",
        description: "Random Teleport Snail uses a homing beacon to get more accurate teleports over time.",
        price: 20_000n,
        order: 1,
        mazeType: "random-teleport",
        showAfter: 25,
    },
    "advanced-homing-beacon": {
        name: "Advanced Homing Beacon",
        description: "Random Teleport Snail upgrades its homing beacon to get even more accurate teleports.",
        icon: "🔊",
        price: 1_000_000n,
        order: 2,
        mazeType: "random-teleport",
        showAfter: 50
    },
    "population-boom": {
        name: "Population Boom",
        description: "A recent population boom has lead to larger generations of Learning Snails.",
        icon: "👥",
        price: 10_000n,
        order: 0,
        mazeType: "learning",
        showAfter: 5
    },
    "uranium": {
        name: "Uranium Mine",
        description: "A nearby uranium mine leads to Learning Snails moving faster and having a higher mutation rate.",
        icon: "☢️",
        price: 500_000n,
        order: 1,
        mazeType: "learning",
        showAfter: 25
    },
    "radium": {
        name: "Radium Mine",
        description: "A nearby radium mine leads to Learning Snails moving faster and having a higher mutation rate.",
        icon: "⚛️",
        price: 2_500_000n,
        order: 2,
        mazeType: "learning",
        showAfter: 50
    },
    "left-glove": {
        name: "Left Glove",
        description: "With a glove on its left hand, Hold Left Snail is able to move 20% faster.",
        icon: "🫲",
        price: 50_000n,
        order: 0,
        mazeType: "hold-left",
        showAfter: 5,
    },
    "right-handed-snail": {
        name: "Right Handed Snail",
        description: "Left Handed Snail recruits Right Handed Snail to help explore the maze faster.",
        icon: "👉",
        price: 1_000_000n,
        order: 1,
        mazeType: "hold-left",
        showAfter: 25,
    },
    "right-glove": {
        name: "Right Glove",
        description: "With a glove on its right hand, Hold Right Snail is able to move 20% faster.",
        icon: "🫱",
        price: 500_000n,
        order: 0,
        mazeType: "inverted",
        showAfter: 5,
    },
    "left-handed-snail": {
        name: "Right Handed Snail",
        description: "Right Handed Snail recruits Left Handed Snail to help explore the maze faster.",
        icon: "👈",
        price: 10_000_000n,
        order: 1,
        mazeType: "inverted",
        showAfter: 25,
    },
    "compass": {
        name: "Compass",
        description: "Using a compass, Segment Snail is able to make smarter decisions about where to go.",
        icon: "🧭",
        price: 10_000_000n,
        order: 0,
        mazeType: "tremaux",
        showAfter: 5,
    },
    "electromagnet": {
        name: "Electromagnet",
        description: "Segment Snail installs an electromagnet near the goal to make its compass more accurate.",
        icon: "⚡",
        price: 50_000_000n,
        order: 1,
        mazeType: "tremaux",
        showAfter: 25,
    },
    "breadcrumbs": {
        name: "Breadcrumbs",
        description: "The Segment Snail leaves breadcrumbs which allow it to backtrack twice as fast.",
        icon: "🍞",
        price: 500_000_000n,
        order: 2,
        mazeType: "tremaux",
        showAfter: 50,
    },
    "comradery": {
        name: "Comradery",
        description: "RPG Snail gets along better with its party, gains +10% movement speed for each member",
        icon: "🫂",
        price: 50_000_000n,
        order: 0,
        mazeType: "rpg",
        showAfter: 5
    },
    "sidequests": {
        name: "Sidequests",
        description: "RPG Snail picks up any snails it runs into.",
        icon: "🛡️",
        price: 150_000_000n,
        order: 1,
        mazeType: "rpg",
        showAfter: 25
    },
    "recruitment": {
        name: "Recruitment",
        description: "Everyone comes to the RPG snail at once.",
        icon: "⚔️",
        price: 3_000_000_000n,
        order: 2,
        mazeType: "rpg",
        showAfter: 50
    },
    "forward-time-travel": {
        name: "Forward Time Travel",
        description: "Time Travel Snail moves 50% faster through time in the present.",
        icon: "⏲️",
        price: 250_000_000n,
        order: 0,
        mazeType: "time-travel",
        showAfter: 5
    },
    "improved-time-relay": {
        name: "Improved Time Relay",
        description: "Time Travel Snail moves 50% faster through time in the past.",
        icon: "⏰",
        price: 1_000_000_000n,
        order: 1,
        mazeType: "time-travel",
        showAfter: 25
    },
    "time-warp": {
        name: "Time Warp",
        description: "Time Travel Snail comes back to the present instantly.",
        icon: "🕰️",
        price: 20_000_000_000n,
        order: 2,
        mazeType: "time-travel",
        showAfter: 50
    },
    "self-improvement": {
        name: "Self-Improvement",
        description: "Cloning Snails improve themselves with each generation.",
        icon: "🤖",
        price: 2_000_000_000n,
        order: 0,
        mazeType: "clone",
        showAfter: 5
    },
    "singularity": {
        name: "Singularity",
        description: "Cloning Snails approach the singularity.",
        icon: "🌎",
        price: 100_000_000_000n,
        order: 1,
        mazeType: "clone",
        showAfter: 25
    }
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
