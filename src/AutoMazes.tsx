import { Component, createEffect, createSignal, For, onCleanup, onMount, untrack, useContext } from "solid-js";
import { SHOP, ShopContext, ShopKey, ShopListing } from "./ShopProvider";
import { ScoreContext } from "./ScoreProvider";
import { createStoredSignal } from "./utils";
import { latticePostMessage, LATTICE_WORKER_STORE } from "./Game";
import { LatticeWorkerResponse } from "./latticeWorker";

// this saves an insane amount of gc time
let CACHED_BUFFERS: Uint8ClampedArray[] = [];
let cachedImageWidth = 0;
let cachedImageHeight = 0;

let reclaimStatus = 0;

function requestBuffer(width: number, height: number) {
  if (CACHED_BUFFERS.length == 0 || cachedImageWidth != width || cachedImageHeight != height) {
    cachedImageWidth = width;
    cachedImageHeight = height;

    reclaimStatus += 1;

    return new Uint8ClampedArray(width * height * 4);
  }

  // console.log(`cachedImageHeight: ${cachedImageHeight}, cachedImageWidth: ${cachedImageWidth}, reclaimStatus: ${reclaimStatus}`);

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
  let latticeCount = 0;

  const intersectionObserver = new IntersectionObserver(entries => {
    let previouslyHadNoVisible = visibleIndexes.size == 0;

    entries.forEach(entry => {
      let i = +entry.target.getAttribute("index");

      if (entry.isIntersecting) {
        visibleIndexes.add(i);

        if (previouslyHadNoVisible) {
          requestAnimationFrame(renderloop);
          previouslyHadNoVisible = false;
        }
      } else {
        visibleIndexes.delete(i);
      }
    });
  });

  const workerOnMessage = (msg: MessageEvent<LatticeWorkerResponse>) => {
    if (msg.data.type == "render") {
      if (visibleIndexes.size > 0)
        requestAnimationFrame(renderloop);

      for (let page of msg.data.pages) {
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

    } else if (msg.data.type == "lattice-updated") {
      const { height, width, latticeCount: newLatticeCount } = msg.data;

      bufferDimensions.width = width;
      bufferDimensions.height = height;

      let canvases = elements();

      // copy list so it will update properly
      let newElements = [...canvases];

      for (let i = 0; i < canvases.length; i++) {
        canvases[i].width = width;
        canvases[i].height = height;
      }

      // create the correct number of elements
      for (let i = latticeCount; i < newLatticeCount; i++) {
        let newElement: HTMLCanvasElement;

        if (!newElements[i]) {
          newElement = canvasElement(i, intersectionObserver);
          newElements.push(newElement);
        } else {
          newElement = newElements[i];
        }

        newElement.width = width;
        newElement.height = height;
      }

      // remove excess elements
      while (newElements.length > newLatticeCount) newElements.pop();

      setElements(newElements);
    }
  };

  const renderloop = () => {
    let pages = [];
    let buffers = [];

    for (let i of visibleIndexes) {
      let arr = requestBuffer(bufferDimensions.width, bufferDimensions.height);
      pages.push({ page: i, buffer: arr });
      buffers.push(arr.buffer);
    }

    worker.postMessage({ type: "render", pages }, buffers);
  };

  const [elements, setElements] = createSignal<HTMLCanvasElement[]>([]);

  createEffect(() => {
    worker.removeEventListener("message", workerOnMessage);

    // update on key change
    worker = LATTICE_WORKER_STORE[props.key];
    worker.addEventListener("message", workerOnMessage);

    // update on width change
    worker.postMessage({ type: "set-width", width: props.latticeWidth });
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
      <div class="p-8 bg-black text-white font-pixelated flex">
        <select class="bg-black text-xl hover:bg-white hover:text-black transition-colors"
          onChange={(e) => {
            setLatticeWidth(SHOP[e.currentTarget.value as ShopKey].latticeWidth);
            setShownMazeType(e.currentTarget.value as ShopKey);
          }}>
          <For each={shop}>
            {item => <option selected={item.key == shownMazeType()} value={item.key} class="py-4 bg-white text-black">{SHOP[item.key].name}</option>}
          </For>
        </select>

        <div class="text-center ml-auto flex">
          <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeWidth(x => Math.max(x - 1, 1))}>-</button>
          <p class="bg-white text-black p-2">{latticeWidth()}</p>
          <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeWidth(x => Math.min(x + 1, 12))}>+</button>

          {fullscreen() ?
            <button class="ml-4 hover:bg-black hover:text-white text-black bg-white transition-all p-2" onClick={() => {
              document.exitFullscreen();
            }}>fullscreen</button>
            :
            <button class="ml-4 hover:bg-white hover:text-black transition-all p-2" onClick={() => {
              mazeDisplay.requestFullscreen();
            }}>fullscreen</button>
          }
        </div>
      </div>

      {
        <div class="p-2 overflow-auto h-full w-full bg-[#068fef]">
          <SnailLatticeElement key={shownMazeItem().key} count={shownMazeItem().count} latticeWidth={latticeWidth()} />
        </div>
      }
    </div>
  )
};

export default AutoMazes;
