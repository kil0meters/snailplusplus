import { createStoredSignal } from './utils';
import AutoMazes from './AutoMazes';
import SnailMaze from './SnailMaze';
import Shop from './Shop';
import { Component, createEffect, createSignal, onCleanup, onMount, untrack, useContext } from 'solid-js';
import { ScoreContext } from './ScoreProvider';
import { LatticeWorkerMessage, LatticeWorkerResponse } from './latticeWorker';
import LatticeWorker from './latticeWorker.ts?worker';
import { SHOP, ShopContext, ShopKey, SHOP_KEYS } from './ShopProvider';

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

const Game: Component = () => {
    const [score, setScore] = useContext(ScoreContext);
    const updateScore = (newScore: number) => setScore(score() + newScore);
    const [mazeSize, setMazeSize] = createStoredSignal("maze-size", 5);
    const [shop, _] = useContext(ShopContext);

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
            <div class='flex flex-col gap-8 h-full overflow-auto pb-16'>
                <div class='p-8 bg-black flex justify-center'>
                    <span class='text-4xl text-center font-extrabold font-pixelated text-white'>{formattedScore()} fragments</span>
                </div>
                <SnailMaze class='min-h-[70vh] h-full' height={mazeSize()} width={mazeSize()} onScore={(score) => { updateScore(score) }} />
                <AutoMazes />
            </div>
            <Shop />
        </div>
    );
};

export default Game;
