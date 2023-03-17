import { Component, createEffect, createSignal, For, onCleanup, onMount, untrack, useContext } from "solid-js";
import { SHOP, ShopContext, ShopKey, ShopListing } from "./ShopProvider";
import { ScoreContext } from "./ScoreProvider";
import { createStoredSignal, formatNumber } from "./utils";
import { latticePostMessage, LATTICES_FILLED, LATTICE_WORKER_STORE } from "./Game";
import { LatticeWorkerResponse } from "./latticeWorker";
import { render } from "solid-js/web";
import { SnailInfoContext } from "./SnailInfoProvider";
import { AverageContext } from "./AverageProvider";

// this saves an insane amount of gc time
const SnailLatticeElement: Component<ShopListing> = (props) => {
    let container: HTMLDivElement;
    let visibleIndexes = new Set([]);
    let worker = LATTICE_WORKER_STORE[props.key];
    let bufferDimensions = { width: 0, height: 0 };
    let workerMessageQueue: LatticeWorkerResponse[] = [];
    const [focusedIndex, setFocusedIndex] = createSignal<number | null>(null);

    let CACHED_BUFFERS: Uint8ClampedArray[] = [];
    let cachedImageWidth = 0;
    let cachedImageHeight = 0;

    function requestBuffer(width: number, height: number) {
        if (cachedImageWidth != width || cachedImageHeight != height) {
            cachedImageWidth = width;
            cachedImageHeight = height;
            CACHED_BUFFERS.length = 0;
        }

        if (CACHED_BUFFERS.length == 0) {
            return new Uint8ClampedArray(width * height * 4);
        }

        return CACHED_BUFFERS.pop();
    }

    function reclaimBuffer(buffer: Uint8ClampedArray, width: number, height: number) {
        if (width == cachedImageWidth && height == cachedImageHeight) {
            CACHED_BUFFERS.push(buffer);
        }
    }

    function canvasElement(index: number, observer: IntersectionObserver): HTMLCanvasElement {
        let canvas = document.createElement("canvas");
        canvas.setAttribute("index", index.toString());
        canvas.style.width = "100%";
        canvas.style.imageRendering = "pixelated";
        observer.observe(canvas);
        return canvas;
    }

    const intersectionObserver = new IntersectionObserver(entries => {
        entries.forEach(entry => {
            let i = +entry.target.getAttribute("index");

            if (entry.isIntersecting) {
                visibleIndexes.add(i);
            } else {
                visibleIndexes.delete(i);
            }
        });

        if (visibleIndexes.size == 0) {
            CACHED_BUFFERS.length = 0;
        }
    }, { threshold: 0 });

    const workerOnMessage = (msg: MessageEvent<LatticeWorkerResponse>) => {
        workerMessageQueue.push(msg.data);
    };

    let renderLocked = false;

    const renderloop = () => {
        while (workerMessageQueue.length > 0) {
            let data = workerMessageQueue.pop();

            if (data.type == "render") {
                renderLocked = false;

                for (let page of data.pages) {
                    let target = elements()[page.page];

                    if (page.buffer.length != 4 * target.width * target.height) {
                        console.log(page.buffer.length, target.width, target.height);
                        break;
                    }

                    let ctx = target.getContext("2d");

                    let imageData = new ImageData(
                        page.buffer,
                        target.width,
                        target.height,
                    );

                    ctx.putImageData(imageData, 0, 0);

                    reclaimBuffer(imageData.data, target.width, target.height);
                }
            } else if (data.type == "lattice-updated") {
                const { height, width, latticeCount } = data;

                if (latticeCount == 0) continue;

                bufferDimensions.width = width;
                bufferDimensions.height = height;

                let canvases = elements();

                // copy list so it will update properly
                let newElements = [...canvases];

                for (let i = 0; i < newElements.length; i++) {
                    newElements[i].width = width;
                    newElements[i].height = height;
                }

                // create the correct number of elements
                for (let i = newElements.length; i < latticeCount; i++) {
                    let newElement: HTMLCanvasElement;

                    if (!newElements[i]) {
                        newElement = canvasElement(i, intersectionObserver);
                        newElement.width = width;
                        newElement.height = height;

                        newElements.push(newElement);
                    }
                }

                // remove excess elements
                while (newElements.length > latticeCount) newElements.pop();

                setElements(newElements);
            }
        }

        if (!renderLocked) {
            let pages = [];
            let buffers = [];

            for (let i of visibleIndexes) {
                let arr = requestBuffer(bufferDimensions.width, bufferDimensions.height);
                pages.push({ page: i, buffer: arr });
                buffers.push(arr.buffer);
            }

            if (pages.length > 0) {
                worker.postMessage({ type: "render", pages }, buffers);
                renderLocked = true;
            }
        }

        requestAnimationFrame(renderloop);
    };

    const [elements, setElements] = createSignal<HTMLCanvasElement[]>([]);

    const mousemove = (e) => {
        let rect = container.getBoundingClientRect();
        let x = e.clientX - rect.left;
        let y = e.clientY - rect.top;

        let mazeSize = rect.width / props.displayWidth
        let mazeX = Math.floor(x / mazeSize);
        let mazeY = Math.floor(y / mazeSize);
        let index = mazeY * props.displayWidth + mazeX;

        if (mazeY >= 0 && mazeX >= 0 && mazeX < props.displayWidth && index < props.count) {
            setFocusedIndex(index);
        } else {
            setFocusedIndex(null);
        }
    };

    onMount(() => {
        renderloop();
        document.addEventListener("mousemove", mousemove);
    });

    createEffect(() => {
        props.displayWidth;
        props.collapsed;

        worker.removeEventListener("message", workerOnMessage);

        // update on key change
        worker = LATTICE_WORKER_STORE[props.key];
        worker.addEventListener("message", workerOnMessage);

        worker.postMessage({ type: "set-width", width: props.displayWidth });
    });

    const mazeSize = () => container.getBoundingClientRect().width / props.displayWidth;
    const [snailInfo, _setSnailInfo] = useContext(SnailInfoContext)

    // surely linearly searching ~10 elements 4 times isn't a big deal
    const thisSnailInfo = () => snailInfo.find((x) => x.key == props.key);

    const dateFormatter = new Intl.DateTimeFormat("en-US", { month: "2-digit", day: "2-digit", year: "numeric" });
    const snailDate = () => {
        let date = new Date(thisSnailInfo().createdAts[focusedIndex()] * 1000);
        return dateFormatter.format(date);
    };

    let snailInfoElement: HTMLDivElement;

    const snailInfoLeft = () => {
        return (focusedIndex() % props.displayWidth + 0.5) * mazeSize() + container.getBoundingClientRect().left;
    };

    const snailInfoTop = () => {
        let underMaze = (1 + Math.floor(focusedIndex() / props.displayWidth)) * mazeSize() + 4 + container.getBoundingClientRect().top;
        let body = document.querySelector("body").getBoundingClientRect();
        let maxHeight = body.height - 108;

        return Math.min(maxHeight, underMaze);
    }

    return (
        <div ref={container} class={`flex items-center justify-center w-full flex-col`}>
            {focusedIndex() !== null && <div ref={snailInfoElement} class="z-50 flex flex-col bg-black p-4 border-2 border-white shadow-md absolute text-white font-display" style={{
                height: "108px",
                top: `${snailInfoTop()}px`,
                left: `${snailInfoLeft()}px`,
                transform: "translateX(-50%)"
            }}>
                <b>{thisSnailInfo().names[focusedIndex()]}</b>
                <span>solved {thisSnailInfo().solvedCounts[focusedIndex()]} mazes</span>
                <span>purchased {snailDate()}</span>
            </div>}
            {elements()}
        </div>
    );
}

const AutoMazeDisplay: Component<{ key: ShopKey, count: number }> = (props) => {
    let mazeDisplay: HTMLDivElement;
    const [averages, _setAverages] = useContext(AverageContext);
    const [shop, setShop] = useContext(ShopContext);

    const collapsed = () => shop.find((x) => x.key == props.key).collapsed;
    const toggleCollapsed = () => setShop((x) => x.key == props.key, "collapsed", (collapsed) => !collapsed);

    const displayWidth = () => shop.find((x) => x.key == props.key).displayWidth;
    const setDisplayWidth = (func: (input: number) => number) => setShop((x) => x.key == props.key, "displayWidth", func);

    let intervalId: number;

    const [fullscreen, setFullscreen] = createSignal(false);
    const togglefullscreen = () => {
        setFullscreen(f => !f);
    };

    addEventListener('fullscreenchange', togglefullscreen);

    onCleanup(() => {
        clearInterval(intervalId)
        removeEventListener('fullscreenchange', togglefullscreen);
    });

    const averageFps = () => {
        let average = averages.find((x) => x.key == props.key);
        return average.count / average.seconds;
    };

    return (
        <div class="w-full" ref={mazeDisplay}>
            <dt class="sticky z-10 top-0 p-8 text-white bg-black min-h-[128px] my-auto font-diplsay flex font-pixelated overflow-x-auto">
                <div class="flex-col flex font-display">
                    <span class="bg-black text-lg md:text-2xl my-auto font-bold">
                        {SHOP[props.key].name}
                    </span>
                    {averageFps() >= Number.EPSILON && <span>{formatNumber(averageFps(), false)} fragments per second</span>}
                </div>

                <div class="text-center ml-auto flex my-auto">
                    <button class="text-lg font-display font-bold mr-4 px-4 py-2 hover:bg-white hover:text-black transition-colors w-20" onclick={toggleCollapsed}>
                        {collapsed() ? "Show" : "Hide"}
                    </button>

                    <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setDisplayWidth(x => Math.max(x - 1, 1))}>-</button>
                    <p class="bg-white text-black p-2">{displayWidth()}</p>
                    <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setDisplayWidth(x => Math.min(x + 1, 12))}>+</button>

                    {fullscreen() ?
                        <button class="hidden md:block ml-4 hover:bg-black hover:text-white text-black bg-white transition-all p-2" onClick={() => {
                            document.exitFullscreen();
                        }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-fullscreen-exit" viewBox="0 0 16 16">
                                <path d="M5.5 0a.5.5 0 0 1 .5.5v4A1.5 1.5 0 0 1 4.5 6h-4a.5.5 0 0 1 0-1h4a.5.5 0 0 0 .5-.5v-4a.5.5 0 0 1 .5-.5zm5 0a.5.5 0 0 1 .5.5v4a.5.5 0 0 0 .5.5h4a.5.5 0 0 1 0 1h-4A1.5 1.5 0 0 1 10 4.5v-4a.5.5 0 0 1 .5-.5zM0 10.5a.5.5 0 0 1 .5-.5h4A1.5 1.5 0 0 1 6 11.5v4a.5.5 0 0 1-1 0v-4a.5.5 0 0 0-.5-.5h-4a.5.5 0 0 1-.5-.5zm10 1a1.5 1.5 0 0 1 1.5-1.5h4a.5.5 0 0 1 0 1h-4a.5.5 0 0 0-.5.5v4a.5.5 0 0 1-1 0v-4z" />
                            </svg>
                        </button>
                        :
                        <button class="hidden md:block ml-4 hover:bg-white hover:text-black transition-all p-2" onClick={() => {
                            mazeDisplay.requestFullscreen();
                        }}><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-arrows-fullscreen" viewBox="0 0 16 16">
                                <path fill-rule="evenodd" d="M5.828 10.172a.5.5 0 0 0-.707 0l-4.096 4.096V11.5a.5.5 0 0 0-1 0v3.975a.5.5 0 0 0 .5.5H4.5a.5.5 0 0 0 0-1H1.732l4.096-4.096a.5.5 0 0 0 0-.707zm4.344 0a.5.5 0 0 1 .707 0l4.096 4.096V11.5a.5.5 0 1 1 1 0v3.975a.5.5 0 0 1-.5.5H11.5a.5.5 0 0 1 0-1h2.768l-4.096-4.096a.5.5 0 0 1 0-.707zm0-4.344a.5.5 0 0 0 .707 0l4.096-4.096V4.5a.5.5 0 1 0 1 0V.525a.5.5 0 0 0-.5-.5H11.5a.5.5 0 0 0 0 1h2.768l-4.096 4.096a.5.5 0 0 0 0 .707zm-4.344 0a.5.5 0 0 1-.707 0L1.025 1.732V4.5a.5.5 0 0 1-1 0V.525a.5.5 0 0 1 .5-.5H4.5a.5.5 0 0 1 0 1H1.732l4.096 4.096a.5.5 0 0 1 0 .707z" />
                            </svg>
                        </button>
                    }
                </div>
            </dt>

            {!collapsed() &&
                <dd class="p-2 h-full w-full bg-[#068fef]">
                    <SnailLatticeElement key={props.key} count={props.count} displayWidth={displayWidth()} collapsed={collapsed()} />
                </dd>}
        </div >
    );
}

const AutoMazes: Component = () => {
    const [shop, _setShop] = useContext(ShopContext);

    return (
        <dl class="xl:max-h-screen xl:overflow-auto">
            <For each={shop.filter((item) => item.count > 0)}>
                {item => <AutoMazeDisplay key={item.key} count={item.count} />}
            </For>
        </dl>
    )
};

export default AutoMazes;
