import { batch, Component, createEffect, createMemo, createSignal, on, onCleanup, onMount, untrack, useContext } from "solid-js";
import init, { Game } from "../snail-lattice/pkg/snail_lattice";
import { PowerupContext } from "./App";
import { ScoreContext } from "./ScoreProvider";
import { randomSeed } from "./utils";

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

const SnailMaze: Component<SnailMazeProps> = (props) => {
    const [grid, setGrid] = createSignal(new Uint8Array);
    const [score, setScore] = useContext(ScoreContext);
    const [_powerup, setPowerup] = useContext(PowerupContext);

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
            setScore(score() + BigInt(Math.abs(solve)));

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
            game.set_game(1);
            ctx = canvas.getContext("2d");
            prevTime = performance.now();
            requestAnimationFrame(render);
        });
    });

    return (
        <div
            tabindex={-1}
            ref={container}
            class={`flex items-center content-center justify-center outline-0 h-full ${props.class}`}
        >
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
            <canvas
                ref={canvas}
                style={{
                    "image-rendering": "pixelated",
                    "width": `${canvas.width * scale()}px`,
                    "height": `${canvas.height * scale()}px`
                }}
            >
            </canvas>
        </div>
    );
};

export default SnailMaze;
