import { bigint_min, createStoredSignal, formatNumber } from './utils';
import AutoMazes from './AutoMazes';
import SnailMaze from './SnailMaze';
import Shop from './Shop';
import { Component, createEffect, createSignal, Match, onCleanup, onMount, Switch, untrack, useContext } from 'solid-js';
import { produce } from 'solid-js/store';
import { ScoreContext } from './ScoreProvider';
import { LatticeWorkerMessage, LatticeWorkerResponse } from './latticeWorker';
import LatticeWorker from './latticeWorker.ts?worker';
import { SHOP, ShopContext, ShopKey, SHOP_KEYS } from './ShopProvider';
import { PowerupContext } from './App';
import { SnailInfoContext } from './SnailInfoProvider';
import { Upgrade, AutoUpgradeKey, UPGRADES, UpgradesContext } from './UpgradesProvider';
import { AverageContext } from './AverageProvider';
import { NAMES } from "../assets/names";

export const latticePostMessage = (worker: Worker, msg: LatticeWorkerMessage) => worker.postMessage(msg);

export let LATTICES_FILLED = false;
export const LATTICE_WORKER_STORE =
    Object.fromEntries(SHOP_KEYS.map((key) =>
        [key, new LatticeWorker()]
    )) as { [key in ShopKey]: Worker };

function setTickRate(tickRate: number) {
    for (let key in LATTICE_WORKER_STORE) {
        latticePostMessage(LATTICE_WORKER_STORE[key], { type: "set-tick-rate", rate: tickRate });
    }
};

const Determination: Component = () => {
    const [powerup, setPowerup] = useContext(PowerupContext);
    const [widthPercent, setWidthPercent] = createSignal(0);
    let intervalId = 0;

    createEffect(() => {
        clearInterval(intervalId);
        if (powerup.active) {
            setTickRate(powerup.multiplier);

            intervalId = setInterval(() => {
                if (powerup.end < new Date()) {
                    let newPowerup = { ...powerup };
                    newPowerup.active = false;
                    setPowerup(newPowerup);
                }

                let now = new Date();

                let percent = 100 - (now.getTime() - powerup.start.getTime()) / (powerup.end.getTime() - powerup.start.getTime()) * 100;
                setWidthPercent(percent);
            }, 50);
        } else {
            setTickRate(1);
        }
    });

    return <>
        {powerup.active ?
            <div class='z-50 flex flex-col gap-4 shadow-md absolute text-white font-pixelated bg-black border-4 border-white bottom-4 max-w-md left-0 right-0 mx-auto p-4'>
                <span>
                    Your snails are filled with determination. They move {powerup.multiplier}x faster for {Math.floor((powerup.end.getTime() - powerup.start.getTime()) / 1000)} seconds.
                </span>

                <div class='bg-white rounded h-4' style={{
                    width: `${widthPercent()}%`
                }}></div>
            </div> : <></>}
    </>;
}

function setUpgradeNumbers(upgrades: Upgrade[]) {
    let upgradeNumbers = new Map<ShopKey, number>();

    let metaSnailUpgrades = 0;

    for (let i = 0; i < upgrades.length; i++) {
        let upgrade = UPGRADES[upgrades[i].key];
        if (upgrades[i].owned && upgrade.mazeType != "manual") {
            metaSnailUpgrades |= 1 << (3 * SHOP_KEYS.indexOf(upgrade.mazeType) + upgrade.order);

            if (!upgradeNumbers[upgrade.mazeType]) {
                upgradeNumbers[upgrade.mazeType] = 1 << upgrade.order;
            } else {
                upgradeNumbers[upgrade.mazeType] |= 1 << upgrade.order;
            }
        }
    }

    for (let key of SHOP_KEYS) {
        if (key != "meta")
            latticePostMessage(LATTICE_WORKER_STORE[key], { type: "set-upgrades", upgrades: upgradeNumbers[key] || 0 });
    }

    latticePostMessage(LATTICE_WORKER_STORE["meta"], { type: "set-upgrades", upgrades: metaSnailUpgrades });
}

const Game: Component = () => {
    const [score, setScore] = useContext(ScoreContext);
    const [_snailInfo, setSnailInfo] = useContext(SnailInfoContext);
    const updateScore = (newScore: bigint) => setScore(score() + newScore);
    const [mazeSize, setMazeSize] = createStoredSignal("maze-size", 5);
    const [shop, _] = useContext(ShopContext);
    const [powerup, setPowerup] = useContext(PowerupContext);
    const [menuShown, setMenuShown] = createSignal(false);
    const [upgrades, _setUpgrades] = useContext(UpgradesContext);
    const [averages, setAverages] = useContext(AverageContext);

    const [displayedScore, setDisplayedScore] = createSignal(score());

    const setScoreListener = (event: MessageEvent<LatticeWorkerResponse>) => {
        let msg = event.data;
        if (msg.type === "score") { // idk why type inferrence doesn't work here
            let addedScore = SHOP[msg.mazeType].baseMultiplier * BigInt(msg.score);
            setScore(oldScore => oldScore + addedScore);
            setSnailInfo(
                (info) => info.key == msg.mazeType,
                "solvedCounts",
                produce((solvedCounts) => {
                    for (let i = 0; i < msg.solves.length; i += 2) {
                        solvedCounts[msg.solves[i]] += msg.solves[i + 1];
                    }
                })
            );

            setAverages(
                msg.mazeType,
                (count) => count + Number(addedScore),
            );
        }
    };

    onMount(() => {
        shop.forEach(({ key, count }) => {
            LATTICE_WORKER_STORE[key].postMessage({ type: "setup", mazeType: key });
            LATTICE_WORKER_STORE[key].postMessage({ type: "alter", diff: count });
            LATTICE_WORKER_STORE[key].addEventListener("message", setScoreListener)

            // initialize each snail info with the correct amount of data if it doesn't already exist
            setSnailInfo(
                (info) => info.key == key,
                produce((info) => {
                    while (info.names.length < count) {
                        info.names.push(NAMES[Math.floor(Math.random() * NAMES.length)]);
                        info.createdAts.push(Math.floor(Date.now() / 1000));
                        info.solvedCounts.push(0);
                    }
                })
            );
        });
    });

    onCleanup(() => {
        shop.forEach(({ key }) =>
            LATTICE_WORKER_STORE[key].terminate()
        );
    })

    createEffect(() => setUpgradeNumbers(upgrades));


    createEffect(() => {
        let difference = score() - untrack(displayedScore);
        let prev = new Date();

        if (difference < 0) {
            setDisplayedScore(score());
            return;
        }

        const animate = () => {
            let now = new Date();
            let dt = BigInt(now.valueOf() - prev.valueOf());
            setDisplayedScore(bigint_min(displayedScore() + difference * dt / 1000n, score()));

            if (displayedScore() != score()) {
                requestAnimationFrame(animate);
            }
        };

        requestAnimationFrame(animate);
    });

    const fragmentsPerSecond = () => {
        let totalCount = 0;
        let seconds = averages[0].seconds;

        for (let i = 0; i < averages.length; i++) {
            totalCount += averages[i].count;
        }

        return totalCount / seconds;
    };

    return <>
        <Determination />

        <div class='grid md:grid-rows-1 md:grid-cols-[minmax(0,auto)_minmax(0,450px)] xl:overflow-hidden md:max-h-screen bg-snailfg'>
            <div class='xl:grid xl:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] gap-8 xl:gap-0 xl:pb-0 md:overflow-auto'>
                <div class='xl:min-h-0 xl:border-r-2 border-black flex flex-col overflow-hidden'>
                    <div class='p-8 bg-black flex flex-col justify-center h-[128px] min-h-[128px] content-center text-white font-display'>
                        <span class='text-3xl text-center font-extrabold my-auto'>{formatNumber(displayedScore(), false)} fragments</span>
                        {fragmentsPerSecond() >= Number.EPSILON && <span class='text-lg text-center'>{formatNumber(fragmentsPerSecond(), true)} fragments per second</span>}
                        <button
                            class='font-display select-none font-bold bg-white text-black absolute md:hidden right-5 my-auto mt-2 px-4 py-2 rounded-md shadow-md border-2 border-black hover:bg-neutral-200 transition-colors'
                            onclick={() => setMenuShown((shown) => !shown)}
                        >menu</button>
                    </div>
                    <SnailMaze class='my-auto' />
                </div>
                <AutoMazes />
            </div>
            <Shop class={`${menuShown() ? '' : 'hidden'}`} />
        </div>
    </>;
};

// new snail ideas:
// - omnipotent snail: rearranges the maze to walk directly to the end
// - conway's snail: clones itself according to the rules of conway's game of life
// - maze snail: each tile is a mini maze that solves itself

export default Game;
