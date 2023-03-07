import { bigint_min, createStoredSignal } from './utils';
import AutoMazes from './AutoMazes';
import SnailMaze from './SnailMaze';
import Shop from './Shop';
import { Component, createEffect, createSignal, onCleanup, onMount, untrack, useContext } from 'solid-js';
import { produce } from 'solid-js/store';
import { ScoreContext } from './ScoreProvider';
import { LatticeWorkerMessage, LatticeWorkerResponse } from './latticeWorker';
import LatticeWorker from './latticeWorker.ts?worker';
import { SHOP, ShopContext, ShopKey, SHOP_KEYS } from './ShopProvider';
import { PowerupContext } from './App';
import { SnailInfoContext } from './SnailInfoProvider';
import { Upgrade, UpgradeKey, UPGRADES, UpgradesContext } from './UpgradesProvider';
import { AverageContext } from './AverageProvider';
import { NAMES } from "../assets/names";

export const latticePostMessage = (worker: Worker, msg: LatticeWorkerMessage) => worker.postMessage(msg);

export let LATTICES_FILLED = false;
export const LATTICE_WORKER_STORE: { [key in ShopKey]: Worker } = {
    "clone": new LatticeWorker(),
    "hold-left": new LatticeWorker(),
    "inverted": new LatticeWorker(),
    "learning": new LatticeWorker(),
    "meta": new LatticeWorker(),
    "random-teleport": new LatticeWorker(),
    "random-walk": new LatticeWorker(),
    "rpg": new LatticeWorker(),
    "time-travel": new LatticeWorker(),
    "tremaux": new LatticeWorker(),
};

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
            <div class='z-50 flex flex-col gap-4 shadow-md absolute text-white font-pixelated bg-black border-4 border-white top-4 max-w-md left-0 right-0 mx-auto p-4'>
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
        if (upgrades[i].owned) {
            let upgrade = UPGRADES[upgrades[i].key];

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
                (average) => average.key == msg.mazeType,
                "count",
                (count) => count + Number(addedScore),
            );
        }
    };

    setInterval(() => {
        setAverages(
            () => true,
            "seconds",
            (seconds) => seconds + 1,
        );
    }, 1000);

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

    const fmt = new Intl.NumberFormat('en', { notation: "compact", maximumSignificantDigits: 3, minimumSignificantDigits: 3 });
    const formattedScore = () => fmt.format(displayedScore());

    const fragmentsPerSecond = () => {
        let totalCount = 0;
        let seconds = averages[0].seconds;

        for (let i = 0; i < averages.length; i++) {
            totalCount += averages[i].count;
        }

        return fmt.format(totalCount / seconds);
    };

    return <>
        <Determination />
        <div class='grid md:grid-rows-1 md:grid-cols-[minmax(0,auto)_minmax(0,450px)] xl:overflow-hidden md:max-h-screen bg-[#068fef]'>
            <div class='flex flex-col xl:grid xl:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] gap-8 xl:gap-0 pb-16 xl:pb-0 md:overflow-auto'>
                <div class='md:min-h-[50vh] xl:min-h-0 xl:border-r-2 border-black flex flex-col max-h-full overflow-hidden'>
                    <div class='p-8 bg-black flex flex-col justify-center h-[128px] content-center text-white font-display'>
                        <span class='text-3xl text-center font-extrabold my-auto'>{formattedScore()} fragments</span>
                        <span class='text-lg text-center'>{fragmentsPerSecond()} fragments/second</span>
                        <button
                            class='font-display select-none font-bold bg-white text-black absolute md:hidden right-5 my-auto mt-2 px-4 py-2 rounded-md shadow-md border-2 border-black hover:bg-neutral-200 transition-colors'
                            onclick={() => setMenuShown((shown) => !shown)}
                        >menu</button>
                    </div>
                    <SnailMaze class='my-auto' height={mazeSize()} width={mazeSize()} onScore={(score, isSpecial) => {
                        updateScore(BigInt(score));
                        setMazeSize(Math.max(Math.floor(Math.random() * 15), 5));

                        if (isSpecial) {
                            let calculatedBoost = Math.max(Math.floor(Math.sqrt(Math.random() * 100)), 2);
                            let boostDuration = Math.max(Math.floor(Math.sqrt(Math.random() * 1000)), 4);
                            let start = new Date();
                            let end = new Date(start);
                            end.setSeconds(end.getSeconds() + boostDuration);

                            setPowerup({
                                active: true,
                                start,
                                end,
                                multiplier: calculatedBoost,
                            });
                        }
                    }} />
                </div>
                <AutoMazes />
            </div>
            <Shop class={`z-30 md:flex ${menuShown() ? '' : 'hidden'}`} />
        </div>
    </>;
};

export default Game;
