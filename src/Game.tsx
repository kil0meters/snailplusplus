import { createStoredSignal } from './utils';
import AutoMazes from './AutoMazes';
import SnailMaze from './SnailMaze';
import Shop from './Shop';
import { Component, createEffect, createSignal, onCleanup, onMount, untrack, useContext } from 'solid-js';
import { ScoreContext } from './ScoreProvider';
import { LatticeWorkerMessage, LatticeWorkerResponse } from './latticeWorker';
import LatticeWorker from './latticeWorker.ts?worker';
import { SHOP, ShopContext, ShopKey, SHOP_KEYS } from './ShopProvider';
import { PowerupContext } from './App';

export const latticePostMessage = (worker: Worker, msg: LatticeWorkerMessage) => worker.postMessage(msg);

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
        console.log("this code is run");
        console.log(`${JSON.stringify(powerup)}`);
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
            <div class='flex flex-col gap-4 shadow-md absolute text-white font-pixelated bg-black border-4 border-white top-4 max-w-md left-0 right-0 mx-auto z-10 p-4'>
                <span>
                    Your snails are filled with determination. They move {powerup.multiplier}x faster for {Math.floor((powerup.end.getTime() - powerup.start.getTime()) / 1000)} seconds.
                </span>

                <div class='bg-white rounded h-4' style={{
                    width: `${widthPercent()}%`
                }}></div>
            </div> : <></>}
    </>;
}

const Game: Component = () => {
    const [score, setScore] = useContext(ScoreContext);
    const updateScore = (newScore: number) => setScore(score() + newScore);
    const [mazeSize, setMazeSize] = createStoredSignal("maze-size", 5);
    const [shop, _] = useContext(ShopContext);
    const [powerup, setPowerup] = useContext(PowerupContext);
    const [shown, setShown] = createSignal(false);

    const [displayedScore, setDisplayedScore] = createSignal(score());

    const setScoreListener = (event: MessageEvent<LatticeWorkerResponse>) => {
        let msg = event.data;
        if (msg.type == "score") {
            setScore(oldScore => oldScore + (SHOP[msg.mazeType].baseMultiplier * msg.score));
        }
    };

    onMount(() => {
        shop.forEach(({ key, count }) => {
            console.log(key);
            LATTICE_WORKER_STORE[key].postMessage({ type: "setup", mazeType: key });
            LATTICE_WORKER_STORE[key].postMessage({ type: "alter", diff: count });
            LATTICE_WORKER_STORE[key].addEventListener("message", setScoreListener)
        });
    });

    onCleanup(() => {
        shop.forEach(({ key }) =>
            LATTICE_WORKER_STORE[key].terminate()
        );
    })

    createEffect(() => {
        let difference = score() - untrack(displayedScore);
        let prev = new Date();

        if (difference < 0) {
            setDisplayedScore(score());
            return;
        }

        const animate = () => {
            let now = new Date();
            let dt = now.valueOf() - prev.valueOf();
            setDisplayedScore(Math.min(displayedScore() + difference * dt / 1000, score()));

            if (displayedScore() != score()) {
                requestAnimationFrame(animate);
            }
        };

        requestAnimationFrame(animate);
    });

    const fmt = new Intl.NumberFormat('en', { notation: "compact", maximumSignificantDigits: 3, minimumSignificantDigits: 3 });
    const formattedScore = () => fmt.format(displayedScore());

    return (
        <div class='grid grid-cols-[minmax(0,5fr)_minmax(0,3fr)] overflow-hidden bg-[#068fef]'>
            <Determination />
            <div class='flex flex-col gap-8 h-full overflow-auto pb-16'>
                <div class='p-8 bg-black flex justify-center'>
                    <span class='text-4xl text-center font-extrabold font-pixelated text-white'>{formattedScore()} fragments</span>
                </div>
                <SnailMaze class='min-h-[70vh] h-full' height={mazeSize()} width={mazeSize()} onScore={(score, isSpecial) => {
                    updateScore(score);
                    setMazeSize(Math.max(Math.floor(Math.random() * 15), 5));

                    if (isSpecial) {
                        let calculatedBoost = Math.floor(Math.sqrt(Math.random() * 100));
                        let boostDuration = Math.floor(Math.sqrt(Math.random() * 1000));
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
                <AutoMazes />
            </div>
            <Shop />
        </div>
    );
};

export default Game;