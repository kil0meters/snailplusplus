import { batch, Component, createEffect, createMemo, createSignal, on, onCleanup, onMount, untrack } from "solid-js";
import snail from '../assets/snail.png';
import goal from '../assets/goal.png';
import { generateMaze } from './utils';
import init, { SnailLattice } from "snail-lattice";
import drawMaze from "./drawMaze";

interface SnailMazeProps {
    onScore: (score: number, isSpecial: boolean) => void;
    width: number;
    class?: string;
    height: number;
    animate?: boolean;
};

export const SNAIL_MOVEMENT_TIME = 150;

const SnailMaze: Component<SnailMazeProps> = (props) => {
    const [grid, setGrid] = createSignal(new Uint8Array);
    let canvas: HTMLCanvasElement | undefined;
    let ctx: CanvasRenderingContext2D;

    let isVisible = true;
    let isSpecial = false;

    let position = [0, 0];
    let prevPosition = [0, 0];
    let movement = [];

    let lastMovement = 1;

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

    let lastMoved = performance.now();
    const renderLoop = () => {
        let now = performance.now();
        let cell = grid()[position[1] * props.width + position[0]];

        let timeSinceLastMove = now - lastMoved;

        let moved = false;

        if (timeSinceLastMove > SNAIL_MOVEMENT_TIME) {
            // right
            if ((movement[0] & 1) != 0 && (cell & 1) == 0) {
                prevPosition = [...position];
                position[0] += 1;
                moved = true;
            }

            // left
            else if ((movement[0] & 2) != 0 && (cell & 2) == 0) {
                prevPosition = [...position];
                position[0] -= 1;
                moved = true;
            }

            // down
            else if ((movement[0] & 4) != 0 && (cell & 4) == 0) {
                prevPosition = [...position];
                position[1] += 1;
                moved = true;
            }

            // up
            else if ((movement[0] & 8) != 0 && (cell & 8) == 0) {
                prevPosition = [...position];
                position[1] -= 1;
                moved = true;
            }
        }

        if (moved) {
            lastMoved = now;
            lastMovement = movement[0];

            if (position[0] == props.width - 1 && position[1] == props.height - 1) {
                setTimeout(() => {
                    props.onScore(props.width * props.height, isSpecial);
                    // see if next maze will be special. about 1/10
                    isSpecial = Math.random() > 0.9 ? true : false;

                    generateMaze(props.width, props.height, (maze) => {
                        setGrid(maze);
                        position = [0, 0];
                    });
                }, SNAIL_MOVEMENT_TIME / 2);
            }
        }

        draw()();
        requestAnimationFrame(renderLoop);
    };
    requestAnimationFrame(renderLoop);

    createEffect(() => {
        if (!canvas) return;

        canvas.width = props.width * 10 + 1;
        canvas.height = props.height * 10 + 1;

        let ctxi = canvas.getContext("2d", { alpha: false });
        ctxi.fillStyle = "#110aef";
        ctxi.imageSmoothingEnabled = false;
        ctxi.fillRect(0, 0, canvas.width, canvas.height);

        ctx = ctxi;
    });

    const snailImage = new Image;
    snailImage.src = snail;

    snailImage.onload = () => {
        requestAnimationFrame(draw);
    };

    const goalImage = new Image;
    goalImage.src = goal;

    goalImage.onload = () => {
        requestAnimationFrame(draw);
    };

    // start on mount
    createEffect(() => {
        generateMaze(props.width, props.height, (maze) => {
            setGrid(maze);
        });
    });


    const gridCanvas = document.createElement('canvas');

    // render grid whenever grid changes
    createEffect(() => {
        if (grid().length != props.width * props.height) return;

        drawMaze(gridCanvas, grid(), props.width, props.height, isSpecial);
    });

    function drawImage(image: HTMLImageElement, x: number, y: number, rotation: number, flip?: boolean) {
        ctx.setTransform(1, 0, 0, 1, Math.floor(x), Math.floor(y)); // sets scale and origin
        if (flip) {
            ctx.scale(-1, 1);
        }
        ctx.rotate(rotation);
        if (animation) {
            ctx.drawImage(image, 8, 0, 8, 8, -4, -4, 8, 8);
        } else {
            ctx.drawImage(image, 0, 0, 8, 8, -4, -4, 8, 8);
        }
    }

    const draw = createMemo(() => () => {
        if (!isVisible) return;

        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.drawImage(gridCanvas, 0, 0);

        let now = performance.now();
        let timeSinceLastMove = now - lastMoved;

        let x: number, y: number;

        if (timeSinceLastMove < SNAIL_MOVEMENT_TIME) {
            x = prevPosition[0] + (position[0] - prevPosition[0]) * (timeSinceLastMove / SNAIL_MOVEMENT_TIME);
            y = prevPosition[1] + (position[1] - prevPosition[1]) * (timeSinceLastMove / SNAIL_MOVEMENT_TIME);
        } else {
            x = position[0];
            y = position[1];
        }

        // right
        if ((lastMovement & 1) != 0) {
            drawImage(
                snailImage,
                x * 10 + 6,
                y * 10 + 6,
                0
            );
        }

        // left
        else if ((lastMovement & 2) != 0) {
            drawImage(
                snailImage,
                x * 10 + 5,
                y * 10 + 6,
                0,
                true
            );
        }

        // down
        else if ((lastMovement & 4) != 0) {
            drawImage(
                snailImage,
                x * 10 + 5,
                y * 10 + 6,
                Math.PI / 2
            );
        }

        // up
        else if ((lastMovement & 8) != 0) {
            drawImage(
                snailImage,
                x * 10 + 6,
                y * 10 + 5,
                3 * Math.PI / 2
            );
        }

        drawImage(
            goalImage,
            props.width * 10 - 4,
            props.height * 10 - 4,
            0
        );
    });

    // main animation loop
    let animation = false;
    setInterval(() => {
        animation = !animation;
    }, 500);

    let container: HTMLDivElement;
    const [scale, setScale] = createSignal(1);

    const updateScale = () => {
        const scaleX = container.clientWidth / canvas.width;
        const scaleY = container.clientHeight / canvas.height;
        setScale(Math.floor(Math.min(scaleX, scaleY)));
        // console.log(scaleX);
        // setScale(Math.floor(scaleX));
    }

    createEffect(() => {
        props.height;
        props.width;

        updateScale();
    });

    onMount(() => {
        updateScale();

        const resizeObserver = new ResizeObserver(() => {
            updateScale();
        });

        resizeObserver.observe(container);

        const intersectionObserver = new IntersectionObserver(entries => {
            isVisible = entries[0].isIntersecting;
        }, { threshold: [0] });

        intersectionObserver.observe(container);

        document.addEventListener("keydown", keyPressed);
        document.addEventListener("keyup", keyReleased);
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
                width={props.width * 10 + 1}
                height={props.height * 10 + 1}
                style={{
                    "image-rendering": "pixelated",
                    "width": `${(props.width * 10 + 1) * scale()}px`,
                    "height": `${(props.width * 10 + 1) * scale()}px`
                }}
            >
            </canvas>
        </div>
    );
};

export default SnailMaze;
