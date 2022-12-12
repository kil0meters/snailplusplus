import { Component, createEffect, createSignal, For, onCleanup, untrack, useContext } from "solid-js";
import { SHOP, ShopContext, ShopKey, ShopListing } from "./ShopProvider";
import init, { CloneLattice, HoldLeftLattice, RandomTeleportLattice, RandomWalkLattice, TimeTravelLattice, TremauxLattice, LearningLattice } from "snail-lattice";
import { ScoreContext } from "./ScoreProvider";
import { createStoredSignal } from "./utils";

// this saves an insane amount of gc time
let CACHED_IMAGE: ImageData;
function requestBuffer(width: number, height: number) {
  if (!CACHED_IMAGE || CACHED_IMAGE.width != width || CACHED_IMAGE.height != height) {
    CACHED_IMAGE = new ImageData(
      new Uint8ClampedArray(width * height * 4),
      width,
      height
    );

  }

  return CACHED_IMAGE
}

// see lattice.rs
interface SnailLattice {
  alter: (size: number) => void;
  tick: (dt: number) => number;
  render: (buffer: Uint8Array, index: number, count: number) => void;
  count: () => number;
  get_dimensions: (count: number) => Uint32Array;
  set_width: (width: number) => void;
}

// This class stores an array of SnailLattices, and manages the web worker
// threads therein. Using intersection observers we can only render mazes that
// are in view, dynamically creating and removing canvases based on the
// viewport.
class LatticeList<T extends SnailLattice> {
  lattice: T;
  baseMultiplier: number;
  width: number;
  prevTick: number;

  get count(): number {
    return Math.ceil(this.lattice.count() / this.pageSize);
  }

  get pageSize(): number {
    return this.width * 4;
  }

  constructor(lattice: T, baseMultiplier: number, width: number) {
    this.lattice = lattice;
    this.width = width;
    this.prevTick = performance.now();
    this.baseMultiplier = baseMultiplier;
  }

  getDimensions(): Uint32Array {
    return this.lattice.get_dimensions(this.pageSize);
  }

  // add maze to lattice
  push() {
    this.lattice.alter(1);
  }

  // remove maze from lattice
  pop() {
    this.lattice.alter(-1);
  }

  // tick everything
  tick(): number {
    let now = performance.now();
    let dt = Math.floor((now - this.prevTick) * 1000);
    this.prevTick = performance.now();

    return this.lattice.tick(dt);
  }

  render(page: number, canvas: HTMLCanvasElement) {
    if (!canvas) return;

    let ctx = canvas.getContext("2d", { alpha: true });
    let imageData = requestBuffer(canvas.width, canvas.height);

    // @ts-ignore -- wasm-bindgen limitation, can't specify uint8clamped array
    // in the type signature easily
    this.lattice.render(imageData.data, page * this.pageSize, this.pageSize);

    ctx.putImageData(imageData, 0, 0);
  }

  // fucks up if it doesn't divide evenly right now
  setWidth(width: number) {
    this.width = width;
    this.lattice.set_width(this.width);
  }
};

function randomSeed(): number {
  return self.crypto.getRandomValues(new Uint16Array(1))[0];
}

export let LATTICE_STORE: { [key in ShopKey]: LatticeList<SnailLattice> } | undefined;

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

  const renderloop = () => {
    let lattice = LATTICE_STORE[props.key];

    for (let i of visibleIndexes) {
      let el = elements()[i];

      lattice.tick();
      lattice.render(i, el);
    }

    if (visibleIndexes.size > 0)
      requestAnimationFrame(renderloop);
  };

  const [elements, setElements] = createSignal<HTMLCanvasElement[]>([]);

  createEffect(() => {
    // required to get hot reload to work, but not strictly necessary
    if (!LATTICE_STORE)
      return;

    // update on key change
    let lattice = LATTICE_STORE[props.key];

    // update on width change
    LATTICE_STORE[props.key].setWidth(props.latticeWidth);

    let canvases = untrack(elements);

    // clear elements
    let newElements = [...untrack(elements)];

    let [width, height] = lattice.getDimensions();

    for (let i = 0; i < canvases.length; i++) {
      canvases[i].width = width;
      canvases[i].height = height;
    }

    // create the correct number of elements
    for (let i = 0; i < lattice.count; i++) {
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
    while (newElements.length > lattice.count) newElements.pop();

    setElements(newElements);
  });

  createEffect((prev: ShopListing) => {
    if (prev.key != props.key || !LATTICE_STORE) return { ...props };

    let latticeList = LATTICE_STORE[props.key];

    let [width, height] = latticeList.getDimensions();

    for (let i = prev.count; i < props.count; i++) {
      // returns true if new element is created
      if ((i + 1) / latticeList.pageSize > elements().length) {
        setElements([...elements(), canvasElement(elements().length, intersectionObserver)]);
      }

      // check to see if we need to resize the lattice
      let lastElement = elements()[elements().length - 1];

      lastElement.width = width;
      lastElement.height = height;
    }

    // remove elements
    for (let i = props.count; i < prev.count; i++) {
      latticeList.pop();
    }

    return { ...props };
  }, { ...props });

  return (
    <div ref={container} class={`flex items-center justify-center w-full flex-col`}>
      {elements()}
    </div>
  );
}

const AutoMazes: Component = () => {
  const [shop, _setShop] = useContext(ShopContext);
  const [initialized, setInitialized] = createSignal(false);
  const [score, setScore] = useContext(ScoreContext);

  let intervalId: number;

  // initialize lattice store
  init().then(() => {
    LATTICE_STORE = {
      "random-walk": new LatticeList(new RandomWalkLattice(8, randomSeed()), 1, 8),
      "random-teleport": new LatticeList(new RandomTeleportLattice(5, randomSeed()), 1, 5),
      "hold-left": new LatticeList(new HoldLeftLattice(4, randomSeed()), 1, 4),
      "tremaux": new LatticeList(new TremauxLattice(3, randomSeed()), 1, 3),
      "time-travel": new LatticeList(new TimeTravelLattice(3, randomSeed()), 1, 3),
      "learning": new LatticeList(new LearningLattice(3, randomSeed()), 1, 3),
      "clone": new LatticeList(new CloneLattice(2, randomSeed()), 1, 2),
    };

    // add initial count to each elements
    for (let [key, lattice] of Object.entries(LATTICE_STORE)) {
      let count = shop.find(x => x.key == key).count;

      lattice.lattice.alter(count);
    }

    setInitialized(true);

    // tick every 100 milliseconds
    intervalId = setInterval(() => {
      let newScore = 0;
      for (let [_, lattice] of Object.entries(LATTICE_STORE)) {
        newScore += lattice.tick();

        setScore(score() + newScore);
      }
    }, 100);
  });

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

      {initialized() &&
        <div class="p-2 overflow-auto h-full w-full bg-[#068fef]">
          <SnailLatticeElement key={shownMazeItem().key} count={shownMazeItem().count} latticeWidth={latticeWidth()} />
        </div>
      }
    </div>
  )
};

export default AutoMazes;
