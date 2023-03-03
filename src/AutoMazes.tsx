import { Component, createEffect, createSignal, For, onCleanup, onMount, untrack, useContext } from "solid-js";
import { SHOP, ShopContext, ShopKey, ShopListing } from "./ShopProvider";
import { ScoreContext } from "./ScoreProvider";
import { createStoredSignal } from "./utils";
import { latticePostMessage, LATTICE_WORKER_STORE } from "./Game";
import { LatticeWorkerResponse } from "./latticeWorker";
import { render } from "solid-js/web";

// this saves an insane amount of gc time
let CACHED_BUFFERS: Uint8ClampedArray[] = [];
let cachedImageWidth = 0;
let cachedImageHeight = 0;

function requestBuffer(width: number, height: number) {
    if (CACHED_BUFFERS.length == 0 || cachedImageWidth != width || cachedImageHeight != height) {
        cachedImageWidth = width;
        cachedImageHeight = height;

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

const SnailLatticeElement: Component<ShopListing & { latticeWidth: number }> = (props) => {
    let container: HTMLDivElement;
    let visibleIndexes = new Set([]);
    let worker = LATTICE_WORKER_STORE[props.key];
    let bufferDimensions = { width: 0, height: 0 };
    let workerMessageQueue: LatticeWorkerResponse[] = [];

    const intersectionObserver = new IntersectionObserver(entries => {
        // let previouslyHadNoVisible = visibleIndexes.size == 0;

        entries.forEach(entry => {
            let i = +entry.target.getAttribute("index");

            if (entry.isIntersecting) {
                visibleIndexes.add(i);

                // if (previouslyHadNoVisible) {
                //     requestAnimationFrame(renderloop);
                //     previouslyHadNoVisible = false;
                // }
            } else {
                visibleIndexes.delete(i);
            }
        });
    }, { threshold: 0 });

    const workerOnMessage = (msg: MessageEvent<LatticeWorkerResponse>) => {
        workerMessageQueue.push(msg.data);
    };

    const renderloop = () => {
        let pages = [];
        let buffers = [];

        for (let i of visibleIndexes) {
            let arr = requestBuffer(bufferDimensions.width, bufferDimensions.height);
            pages.push({ page: i, buffer: arr });
            buffers.push(arr.buffer);
        }

        if (pages.length > 0) {
            worker.postMessage({ type: "render", pages }, buffers);
        }

        requestAnimationFrame(renderloop);

        while (workerMessageQueue.length > 0) {
            let data = workerMessageQueue.pop();
            if (data.type == "render") {
                // if (visibleIndexes.size > 0)
                //     requestAnimationFrame(renderloop);

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
    };

    let firstRender = true;

    onMount(() => {
        renderloop();
    });

    const [elements, setElements] = createSignal<HTMLCanvasElement[]>([]);

    createEffect(() => {
        worker.removeEventListener("message", workerOnMessage);

        // update on key change
        worker = LATTICE_WORKER_STORE[props.key];
        worker.addEventListener("message", workerOnMessage);

        if (firstRender) {
            firstRender = false;
        } else {
            worker.postMessage({ type: "set-width", width: props.latticeWidth });
        }
    });

    return (
        <div ref={container} class={`flex items-center justify-center w-full flex-col`}>
            {elements()}
        </div>
    );
}

const AutoMazes: Component = () => {
    const [shop, _setShop] = useContext(ShopContext);
    const [score, setScore] = useContext(ScoreContext);

    let intervalId: number;

    const togglefullscreen = () => {
        setFullscreen(f => !f);
    };

    addEventListener('fullscreenchange', togglefullscreen);

    onCleanup(() => {
        clearInterval(intervalId)
        removeEventListener('fullscreenchange', togglefullscreen);
    });

    let mazeDisplay: HTMLDivElement;
    const [shownMazeType, setShownMazeType] = createStoredSignal<ShopKey>("shown-maze", "random-walk");

    const shownMazeItem = () => shop.find(el => el.key == shownMazeType());

    const [latticeWidth, setLatticeWidth] = createSignal(SHOP[shownMazeType()].latticeWidth);

    const [fullscreen, setFullscreen] = createSignal(false);

    return (
        <div class="w-full flex flex-col" ref={mazeDisplay}>
            <div class="p-8 text-white bg-black min-h-[128px] my-auto font-diplsay flex font-pixelated">
                <select class="bg-black text-2xl my-auto p-2 hover:bg-white hover:text-black transition-colors font-display font-bold"
                    onChange={(e) => {
                        setLatticeWidth(SHOP[e.currentTarget.value as ShopKey].latticeWidth);
                        setShownMazeType(e.currentTarget.value as ShopKey);
                    }}>
                    <For each={shop}>
                        {item => <option selected={item.key == shownMazeType()} value={item.key} class="py-4 bg-white text-black">{SHOP[item.key].name}</option>}
                    </For>
                </select>

                <div class="text-center ml-auto flex my-auto">
                    <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeWidth(x => Math.max(x - 1, 1))}>-</button>
                    <p class="bg-white text-black p-2">{latticeWidth()}</p>
                    <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeWidth(x => Math.min(x + 1, 12))}>+</button>

                    {fullscreen() ?
                        <button class="ml-4 hover:bg-black hover:text-white text-black bg-white transition-all p-2" onClick={() => {
                            document.exitFullscreen();
                        }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-fullscreen-exit" viewBox="0 0 16 16">
                                <path d="M5.5 0a.5.5 0 0 1 .5.5v4A1.5 1.5 0 0 1 4.5 6h-4a.5.5 0 0 1 0-1h4a.5.5 0 0 0 .5-.5v-4a.5.5 0 0 1 .5-.5zm5 0a.5.5 0 0 1 .5.5v4a.5.5 0 0 0 .5.5h4a.5.5 0 0 1 0 1h-4A1.5 1.5 0 0 1 10 4.5v-4a.5.5 0 0 1 .5-.5zM0 10.5a.5.5 0 0 1 .5-.5h4A1.5 1.5 0 0 1 6 11.5v4a.5.5 0 0 1-1 0v-4a.5.5 0 0 0-.5-.5h-4a.5.5 0 0 1-.5-.5zm10 1a1.5 1.5 0 0 1 1.5-1.5h4a.5.5 0 0 1 0 1h-4a.5.5 0 0 0-.5.5v4a.5.5 0 0 1-1 0v-4z" />
                            </svg>
                        </button>
                        :
                        <button class="ml-4 hover:bg-white hover:text-black transition-all p-2" onClick={() => {
                            mazeDisplay.requestFullscreen();
                        }}><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-arrows-fullscreen" viewBox="0 0 16 16">
                                <path fill-rule="evenodd" d="M5.828 10.172a.5.5 0 0 0-.707 0l-4.096 4.096V11.5a.5.5 0 0 0-1 0v3.975a.5.5 0 0 0 .5.5H4.5a.5.5 0 0 0 0-1H1.732l4.096-4.096a.5.5 0 0 0 0-.707zm4.344 0a.5.5 0 0 1 .707 0l4.096 4.096V11.5a.5.5 0 1 1 1 0v3.975a.5.5 0 0 1-.5.5H11.5a.5.5 0 0 1 0-1h2.768l-4.096-4.096a.5.5 0 0 1 0-.707zm0-4.344a.5.5 0 0 0 .707 0l4.096-4.096V4.5a.5.5 0 1 0 1 0V.525a.5.5 0 0 0-.5-.5H11.5a.5.5 0 0 0 0 1h2.768l-4.096 4.096a.5.5 0 0 0 0 .707zm-4.344 0a.5.5 0 0 1-.707 0L1.025 1.732V4.5a.5.5 0 0 1-1 0V.525a.5.5 0 0 1 .5-.5H4.5a.5.5 0 0 1 0 1H1.732l4.096 4.096a.5.5 0 0 1 0 .707z" />
                            </svg>
                        </button>
                    }
                </div>
            </div>

            <div class="p-2 overflow-auto h-full w-full bg-[#068fef]">
                <SnailLatticeElement key={shownMazeItem().key} count={shownMazeItem().count} latticeWidth={latticeWidth()} />
            </div>
        </div>
    )
};

export default AutoMazes;
