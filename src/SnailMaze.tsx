import { Component, createEffect, createSignal, For, onMount, useContext } from "solid-js";
import init, { Game } from "../snail-lattice/pkg/snail_lattice";
import { PowerupContext } from "./App";
import { ScoreContext } from "./ScoreProvider";
import { UPGRADES, UpgradesContext } from "./UpgradesProvider";
import { createStoredSignal, formatNumber, randomSeed } from "./utils";

interface SnailMazeProps {
    class?: string;
    animate?: boolean;
};

let movement: number[] = [];

const keyPressed = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
        case 'w':
        case 'W':
        case 'ArrowUp':
            e.preventDefault();
            movement.unshift(8);
            break;
        case 'a':
        case 'A':
        case 'ArrowLeft':
            e.preventDefault();
            movement.unshift(2);
            break;
        case 's':
        case 'S':
        case 'ArrowDown':
            e.preventDefault();
            movement.unshift(4);
            break;
        case 'd':
        case 'D':
        case 'ArrowRight':
            e.preventDefault();
            movement.unshift(1);
            break;
    }
};

const keyReleased = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
        case 'w':
        case 'W':
        case 'ArrowUp':
            e.preventDefault();
            movement = movement.filter(x => x != 8);
            break;
        case 'a':
        case 'A':
        case 'ArrowLeft':
            e.preventDefault();
            movement = movement.filter(x => x != 2);
            break;
        case 's':
        case 'S':
        case 'ArrowDown':
            e.preventDefault();
            movement = movement.filter(x => x != 4);
            break;
        case 'd':
        case 'D':
        case 'ArrowRight':
            e.preventDefault();
            movement = movement.filter(x => x != 1);
            break;
    }
};

const MobileControls: Component = () => {
    return (
        <div class="grid z-20 grid-cols-3 grid-rows-3 fixed md:hidden aspect-square right-4 bottom-4 text-5xl w-[196px] h-[196px] opacity-70 select-none">
            <button
                class="col-start-2 row-start-1 bg-white active:bg-neutral-200"
                onmousedown={(e: any) => {
                    e.key = 'ArrowUp';
                    keyPressed(e);
                }}
                onmouseup={(e: any) => {
                    e.key = 'ArrowUp';
                    keyReleased(e);
                }}
                ontouchstart={(e: any) => {
                    e.key = 'ArrowUp';
                    keyPressed(e);
                }}
                ontouchend={(e: any) => {
                    e.key = 'ArrowUp';
                    keyReleased(e);
                }}
            >↑</button>
            <button
                class="col-start-2 row-start-3 bg-white active:bg-neutral-200"
                onmousedown={(e: any) => {
                    e.key = 'ArrowDown';
                    keyPressed(e);
                }}
                onmouseup={(e: any) => {
                    e.key = 'ArrowDown';
                    keyReleased(e);
                }}
                ontouchstart={(e: any) => {
                    e.key = 'ArrowDown';
                    keyPressed(e);
                }}
                ontouchend={(e: any) => {
                    e.key = 'ArrowDown';
                    keyReleased(e);
                }}
            >↓</button>
            <button
                class="col-start-1 row-start-2 bg-white active:bg-neutral-200"
                onmousedown={(e: any) => {
                    e.key = 'ArrowLeft';
                    keyPressed(e);
                }}
                onmouseup={(e: any) => {
                    e.key = 'ArrowLeft';
                    keyReleased(e);
                }}
                ontouchstart={(e: any) => {
                    e.key = 'ArrowLeft';
                    keyPressed(e);
                }}
                ontouchend={(e: any) => {
                    e.key = 'ArrowLeft';
                    keyReleased(e);
                }}
            >←</button>
            <button
                class="col-start-3 row-start-2 bg-white active:bg-neutral-200"
                onmousedown={(e: any) => {
                    e.key = 'ArrowRight';
                    keyPressed(e);
                }}
                onmouseup={(e: any) => {
                    e.key = 'ArrowRight';
                    keyReleased(e);
                }}
                ontouchstart={(e: any) => {
                    e.key = 'ArrowRight';
                    keyPressed(e);
                }}
                ontouchend={(e: any) => {
                    e.key = 'ArrowRight';
                    keyReleased(e);
                }}
            >→</button>
        </div>
    );
};

const SnailMaze: Component<SnailMazeProps> = (props) => {
    const [score, setScore] = useContext(ScoreContext);
    const [upgrades, _setUpgrades] = useContext(UpgradesContext);
    const [_powerup, setPowerup] = useContext(PowerupContext);
    const [gameMode, setGameMode] = createStoredSignal("selected-game", 0);
    const [recentScores, setRecentScores] = createSignal<{ score: bigint, bonus: boolean }[]>([]);

    let game: Game;
    let prevTime: number;
    let buffer = new Uint8ClampedArray(0);
    let container: HTMLDivElement;
    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    const [scale, setScale] = createSignal(1);

    const updateScale = () => {
        const scaleX = container.clientWidth / canvas.width;
        const scaleY = container.clientHeight / canvas.height;
        setScale(Math.min(scaleX, scaleY));
    }

    const render = () => {
        let [width, height] = game.resolution();

        if (buffer.length != width * height * 4) {
            canvas.width = width;
            canvas.height = height;
            buffer = new Uint8ClampedArray(width * height * 4);
            updateScale();
        }

        let now = performance.now();
        let dt = now - prevTime;
        prevTime = now;

        // @ts-ignore: this does work, but due to a wasm-bindgen we cannot make the signature take a Uint8ClampedArray
        let solve = game.render(buffer, new Uint32Array(movement), dt);

        if (solve != 0) {
            let newScore = BigInt(Math.abs(solve))
            setScore(score() + newScore);

            setRecentScores((scores) => [...scores, { score: newScore, bonus: solve < 0 }]);
            setTimeout(() => {
                setRecentScores((scores) => {
                    let newScores = [...scores];
                    newScores.shift();
                    return newScores;
                });
            }, 1000);

            if (solve < 0) {
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
        }

        let imageData = new ImageData(
            buffer,
            width,
            height
        );

        ctx.putImageData(imageData, 0, 0);

        requestAnimationFrame(render);
    };

    createEffect(() => {
        // idk why i need this here, but sure
        gameMode();

        if (game)
            game.set_game(gameMode());
    });

    onMount(() => {
        updateScale();

        const resizeObserver = new ResizeObserver(() => {
            updateScale();
        });

        resizeObserver.observe(container);

        document.addEventListener("keydown", keyPressed);
        document.addEventListener("keyup", keyReleased);

        init().then(() => {
            game = new Game(randomSeed());
            game.set_game(gameMode());
            ctx = canvas.getContext("2d");
            prevTime = performance.now();
            requestAnimationFrame(render);
        });
    });

    return <>
        <MobileControls />

        <div
            tabindex={-1}
            ref={container}
            class={`outline-0 h-full justify-center items-center flex flex-col relative ${props.class}`}
        >
            <For each={recentScores()}>{(score) => {
                return <span class="text-xl text-white drop-shadow-lg font-bold font-display absolute animate-slide-out top-[20%]">{score.bonus ? "Bonus!" : ""} {formatNumber(score.score, false)} fragments</span>
            }}</For>

            <canvas
                class="my-auto max-w-full"
                ref={canvas}
                style={{
                    "image-rendering": "pixelated",
                    "width": `${canvas.width * scale()}px`,
                    "height": `${canvas.height * scale()}px`
                }}
            >
            </canvas>

            {upgrades.find((upgrade) => UPGRADES[upgrade.key].mazeType == "manual" && upgrade.owned) != undefined ?
                <div class="bg-black border-black border-2 text-lg py-4 gap-2 shadow-md w-full">
                    <span class="font-display text-white font-bold px-4">Manual Snail</span>

                    <div class="flex overflow-x-auto">
                        <div class="grid grid-cols-4 grid-flow-col pl-4">
                            <button class="p-2 hover:bg-white aspect-square px-4 text-2xl" onClick={() => setGameMode(0)}>🐌</button>

                            <For each={upgrades.filter((upgrade) => {
                                return upgrade.owned && UPGRADES[upgrade.key].mazeType == "manual";
                            })}>{(upgrade) =>
                                <button class={`p-2 hover:bg-white aspect-square text-2xl transition-colors ${UPGRADES[upgrade.key].order + 1 === gameMode() ? "bg-white" : ""}`} onClick={() => setGameMode(UPGRADES[upgrade.key].order + 1)}>{UPGRADES[upgrade.key].icon}</button>
                                }</For>
                        </div>
                    </div>
                </div>
                :
                <div class="bg-snailfg py-4 w-full"></div>
            }
        </div>
    </>;
};



export default SnailMaze;
