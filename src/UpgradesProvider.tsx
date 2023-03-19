import { Component, createContext, JSX } from "solid-js";
import { SetStoreFunction } from "solid-js/store";
import { ShopKey } from "./ShopProvider";
import { createLocalStore } from "./utils";

export const UpgradesContext = createContext<[Upgrade[], SetStoreFunction<Upgrade[]>]>();

const AUTO_UPGRADE_KEYS = [
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
    "singularity",
    "lax-regulations",
    "nitrogen-deposit",
    "destructive-habits",
    "carbon-fiber-exoskeleton",
    "singing-lessons",
    "microphone",
    "untested-surgery",
    "kinesiology-degree",
    "split-brain",
    "high-speed-connectivity",
    "algorithmic-improvement"
] as const;
export type AutoUpgradeKey = typeof AUTO_UPGRADE_KEYS[number];

const MANUAL_UPGRADE_KEYS = [
    "pacsnail",
    "asteroids",
    "wolfensnail",
    "falling-snails"
] as const;
export type ManualUpgradeKey = typeof MANUAL_UPGRADE_KEYS[number];

export type Upgrade = {
    key: UpgradeKey;
    owned: boolean;
};

export type UpgradeKey = ManualUpgradeKey | AutoUpgradeKey

export type UpgradeListing = {
    name: string;
    icon: string;
    description: string;
    price: bigint;
    order: number; // where in the progression the upgrade is
    mazeType: ShopKey | "manual";
    showAfter: number; // certain number of units of that type
};

const UPGRADES_DEFAULT: Upgrade[] = [...AUTO_UPGRADE_KEYS, ...MANUAL_UPGRADE_KEYS].map((key) => { return { key, owned: false } });

export const UPGRADES: { [key in UpgradeKey]: UpgradeListing } = {
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
        price: 50000n,
        order: 2,
        mazeType: "random-walk",
        showAfter: 50,
    },
    "fusion-reactor": {
        name: "Fusion Reactor",
        icon: "☄️",
        description: "Random Teleport Snail uses a fusion reactor to charge up its teleports 20% faster.",
        price: 2000n,
        order: 0,
        mazeType: "random-teleport",
        showAfter: 5,
    },
    "homing-beacon": {
        name: "Homing Beacon",
        icon: "🔉",
        description: "Random Teleport Snail uses a homing beacon to get more accurate teleports over time.",
        price: 20000n,
        order: 1,
        mazeType: "random-teleport",
        showAfter: 25,
    },
    "advanced-homing-beacon": {
        name: "Advanced Homing Beacon",
        description: "Random Teleport Snail upgrades its homing beacon to get even more accurate teleports.",
        icon: "🔊",
        price: 1000000n,
        order: 2,
        mazeType: "random-teleport",
        showAfter: 50
    },
    "population-boom": {
        name: "Population Boom",
        description: "A recent population boom has lead to larger generations of Learning Snails.",
        icon: "👥",
        price: 10000n,
        order: 0,
        mazeType: "learning",
        showAfter: 5
    },
    "uranium": {
        name: "Uranium Mine",
        description: "A nearby uranium mine leads to Learning Snails moving faster and having a higher mutation rate.",
        icon: "☢️",
        price: 500000n,
        order: 1,
        mazeType: "learning",
        showAfter: 25
    },
    "radium": {
        name: "Radium Mine",
        description: "A nearby radium mine leads to Learning Snails moving faster and having a higher mutation rate.",
        icon: "⚛️",
        price: 2500000n,
        order: 2,
        mazeType: "learning",
        showAfter: 50
    },
    "left-glove": {
        name: "Left Glove",
        description: "With a glove on its left hand, Hold Left Snail is able to move 20% faster.",
        icon: "🫲",
        price: 50000n,
        order: 0,
        mazeType: "hold-left",
        showAfter: 5,
    },
    "right-handed-snail": {
        name: "Right Handed Snail",
        description: "Left Handed Snail recruits Right Handed Snail to help explore the maze faster.",
        icon: "👉",
        price: 1000000n,
        order: 1,
        mazeType: "hold-left",
        showAfter: 25,
    },
    "right-glove": {
        name: "Right Glove",
        description: "With a glove on its right hand, Hold Right Snail is able to move 20% faster.",
        icon: "🫱",
        price: 500000n,
        order: 0,
        mazeType: "inverted",
        showAfter: 5,
    },
    "left-handed-snail": {
        name: "Left Handed Snail",
        description: "Right Handed Snail recruits Left Handed Snail to help explore the maze faster.",
        icon: "👈",
        price: 10000000n,
        order: 1,
        mazeType: "inverted",
        showAfter: 25,
    },
    "compass": {
        name: "Compass",
        description: "Using a compass, Segment Snail is able to make smarter decisions about where to go.",
        icon: "🧭",
        price: 10000000n,
        order: 0,
        mazeType: "tremaux",
        showAfter: 5,
    },
    "electromagnet": {
        name: "Electromagnet",
        description: "Segment Snail installs an electromagnet near the goal to make its compass more accurate.",
        icon: "⚡",
        price: 50000000n,
        order: 1,
        mazeType: "tremaux",
        showAfter: 25,
    },
    "breadcrumbs": {
        name: "Breadcrumbs",
        description: "The Segment Snail leaves breadcrumbs which allow it to backtrack twice as fast.",
        icon: "🍞",
        price: 500000000n,
        order: 2,
        mazeType: "tremaux",
        showAfter: 50,
    },
    "comradery": {
        name: "Comradery",
        description: "RPG Snail gets along better with its party, gains +10% movement speed for each member",
        icon: "🫂",
        price: 50000000n,
        order: 0,
        mazeType: "rpg",
        showAfter: 5
    },
    "sidequests": {
        name: "Sidequests",
        description: "RPG Snail picks up any snails it runs into.",
        icon: "🛡️",
        price: 150000000n,
        order: 1,
        mazeType: "rpg",
        showAfter: 25
    },
    "recruitment": {
        name: "Recruitment",
        description: "Everyone comes to the RPG snail at once.",
        icon: "⚔️",
        price: 3000000000n,
        order: 2,
        mazeType: "rpg",
        showAfter: 50
    },
    "forward-time-travel": {
        name: "Forward Time Travel",
        description: "Time Travel Snail moves 50% faster through time in the present.",
        icon: "⏲️",
        price: 250000000n,
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
    },
    "lax-regulations": {
        name: "Lax Regulations",
        description: "A recently passed explosives reform bill allows the Demolitionist Snail to shorten the fuses on its bombs.",
        icon: "📜",
        price: 15_000_000_000n,
        order: 0,
        mazeType: "demolitionist",
        showAfter: 5
    },
    "nitrogen-deposit": {
        name: "Nitrogen Deposit",
        description: "A nearby nitrogen deposit allows the Demolitionist Snail to plant more bombs.",
        icon: "🧨",
        price: 85_000_000_000n,
        order: 1,
        mazeType: "demolitionist",
        showAfter: 25
    },
    "destructive-habits": {
        name: "Destructive Habits",
        description: "Cognative behavioral therapy allows the Demolitionist Snail to view the havoc it has caused in a new light. Gains speed for each destroyed tile it walks over.",
        icon: "🗑️",
        price: 300_000_000_000n,
        order: 2,
        mazeType: "demolitionist",
        showAfter: 50
    },
    "carbon-fiber-exoskeleton": {
        name: "Carbon Fiber Exoskeleton",
        description: "The Swarm Snails acquire an exoskeleton which increases their flying speed.",
        icon: "🚶",
        price: 500_000_000_000n,
        order: 0,
        mazeType: "flying",
        showAfter: 5
    },
    "singing-lessons": {
        name: "Singing Lessonss",
        description: "The Swarm Snails take singing lessons to attract more members into their swarm.",
        icon: "🧑‍🎤",
        price: 4_000_000_000_000n,
        order: 1,
        mazeType: "flying",
        showAfter: 25
    },
    "microphone": {
        name: "Singing Lessonss",
        description: "The Swarm Snails purchase a microphone to increase the range of their hymmn.",
        icon: "🎤",
        price: 25_000_000_000_000n,
        order: 2,
        mazeType: "flying",
        showAfter: 50
    },
    "untested-surgery": {
        name: "Untested Surgery",
        description: "The Telepathic Snail undergoes an experimental surgery which allows it to move and use its telepathy at the same time.",
        icon: "🏥",
        price: 10_000_000_000_000n,
        order: 0,
        mazeType: "telepathic",
        showAfter: 5
    },
    "kinesiology-degree": {
        name: "Kinesiology Degree",
        description: "The Telepathic Snail goes to college to study kinesiology. With a newfound understanding of snail kinematics, it is able to use its telepathic abilities to move faster.",
        icon: "🎓",
        price: 50_000_000_000_000n,
        order: 1,
        mazeType: "telepathic",
        showAfter: 25
    },
    "split-brain": {
        name: "Split Brain",
        description: "The Telepathic Snail fully separates its brain, allowing it to simultaneously move itself and the goal.",
        icon: "🧠",
        price: 1_000_000_000_000_000n,
        order: 2,
        mazeType: "telepathic",
        showAfter: 50
    },

    "high-speed-connectivity": {
        name: "High Speed Connectivity",
        icon: "📶",
        description: "Automaton Snail installs a new 5G radio tower nearby to allow for faster communication between cells.",
        price: 0n,
        order: 0,
        mazeType: "automaton",
        showAfter: 5
    },

    "algorithmic-improvement": {
        name: "Algorithmic Improvement",
        icon: "🦠",
        description: "Automaton Snail changes its replication method to one that is more effective.",
        price: 0n,
        order: 1,
        mazeType: "automaton",
        showAfter: 25
    },

    //////////////////////////
    // MANUAL MAZE UPGRADES //
    //////////////////////////

    // In this context, showAfter means the number of different mazes you have
    // to have unlocked before the upgrade shows up in the shop

    "pacsnail": {
        name: "Larger Maze",
        icon: "🧀",
        description: "The Manual Snail finds itself in a larger maze with abundant fragments. However, it's not alone.",
        price: 10_000n,
        showAfter: 3,

        order: 0,
        mazeType: "manual",
    },
    "asteroids": {
        name: "Rocket Ship",
        icon: "🚀",
        description: "In search of fragments, The Manual Snail goes to space.",
        price: 5_000_000n,
        order: 1,
        mazeType: "manual",
        showAfter: 6
    },
    "wolfensnail": {
        name: "Dimensional Recombobulator",
        icon: "🔫",
        description: "The Manual Snail alters the fabric of reality, entering the third dimension.",
        price: 1_000_000_000n,
        order: 2,
        mazeType: "manual",
        showAfter: 9
    },
    "falling-snails": {
        name: "Falling Snails",
        icon: "🧱",
        description: "The Manual Snail finds itself mysteriously connected with other snails and falling... falling... falling..other snails and falling... falling... falling...",
        price: 500_000_000_000n,
        order: 3,
        mazeType: "manual",
        showAfter: 12
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
