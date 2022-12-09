import { Component, createEffect, createMemo, createSignal, For, onCleanup, onMount, untrack, useContext } from "solid-js";
import { SHOP, ShopContext, ShopItem, ShopKey, ShopListing } from "./ShopProvider";
import init, { CloneLattice, HoldLeftLattice, RandomTeleportLattice, RandomWalkLattice, TimeTravelLattice, TremauxLattice } from "snail-lattice";
import { ScoreContext } from "./ScoreProvider";

// this saves an insane amount of gc time
let reservedBuffers: Map<number, ImageData[]> = new Map();

function requestBuffer(width: number, height: number) {
  let buffer = reservedBuffers.get(width << 32 + height)?.pop();

  if (buffer) {
    return buffer;
  } else {
    console.log(`creating new buffer of size ${width},${height}`);
    return new ImageData(
      new Uint8ClampedArray(width * height * 4),
      width,
      height
    );
  }
}

function reclaimBuffer(image: ImageData) {
  let size = reservedBuffers.get(image.width << 32 + image.height);
  if (size) {
    size.push(image);
  } else {
    reservedBuffers.set(image.width << 32 + image.height, [image]);
  }
}

// see lattice.rs
interface SnailLattice {
  alter: (size: number) => void;
  tick: (dt: number) => number;
  render: (buffer: Uint8Array) => void;
  count: () => number;
  get_dimensions: () => Uint32Array;
  set_width: (width: number) => void;
}

// This class stores an array of SnailLattices, and manages the web worker
// threads therein. Using intersection observers we can only render mazes that
// are in view, dynamically creating and removing canvases based on the
// viewport.
class LatticeList<T extends SnailLattice> {
  store: T[];
  latticeFactory: () => T;
  baseWidth: number;
  count: number;
  baseMultiplier: number;
  scale: number;

  prevTick: number;

  get width(): number {
    return this.baseWidth * this.scale;
  }

  get maxPerLattice(): number {
    return this.baseWidth * 6;
  }

  constructor(latticeFactory: () => T, baseMultiplier: number, width: number) {
    this.latticeFactory = latticeFactory;
    this.store = [latticeFactory()];
    this.baseWidth = width;
    this.prevTick = performance.now();
    this.count = 1;
    this.baseMultiplier = baseMultiplier;
    this.scale = 1;
  }

  getDimensions(index: number): Uint32Array {
    return this.store[index].get_dimensions();
  }

  // add maze to lattice
  push(): boolean {
    let top = this.store[this.store.length - 1];

    // we cap at 8 rows per lattice
    if (top.count() < this.maxPerLattice) {
      top.alter(1);

      return false;
    } else {
      let newLattice = this.latticeFactory();
      newLattice.set_width(this.width);
      newLattice.alter(1);

      this.store.push(newLattice);
      this.count++;

      return true;
    }
  }

  // remove maze from lattice
  pop() {
    let top = this.store[this.store.length - 1];

    top.alter(-1);

    if (top.count() == 0) {
      this.store.pop();
      this.count--;
    }
  }

  // tick everything
  tick(): number {
    let now = performance.now();
    let dt = Math.floor((now - this.prevTick) * 1000);
    this.prevTick = performance.now();

    let score = this.store
      .map(lattice => Math.floor(lattice.tick(dt) * this.baseMultiplier))
      .reduce((acc, curr) => acc + curr, 0);

    return score;
  }

  render(index: number, canvas: HTMLCanvasElement) {
    if (!canvas) return;

    let lattice = this.store[index];

    if (!lattice) return;

    let ctx = canvas.getContext("2d", { alpha: true });
    let imageData = requestBuffer(canvas.width, canvas.height);

    // @ts-ignore -- wasm-bindgen limitation, can't specify uint8clamped array
    // in the type signature easily
    lattice.render(imageData.data);

    ctx.putImageData(imageData, 0, 0);

    reclaimBuffer(imageData);
  }

  // fucks up if it doesn't divide evenly right now
  setScale(scale: number) {
    this.scale = scale;
    this.store.forEach(lattice => lattice.set_width(this.width));
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

const SnailLatticeElement: Component<ShopListing & { scale: number }> = (props) => {
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

      // only render if we actually have to
      if (el?.height > 0) {
        lattice.tick();
        lattice.render(i, elements()[i]);
      }
    }

    if (visibleIndexes.size > 0)
      requestAnimationFrame(renderloop);
  };

  // set width
  createEffect(() => {
    LATTICE_STORE[props.key].setScale(props.scale);

    let store = LATTICE_STORE[props.key].store;
    let canvases = untrack(elements);

    // resize each lattice
    for (let i = 0; i < canvases.length; i++) {
      if (store[i]) {
        let [width, height] = store[i].get_dimensions();
        canvases[i].width = width;
        canvases[i].height = height;
      }
    }
  })

  const [elements, setElements] = createSignal<HTMLCanvasElement[]>([]);

  createEffect(() => {
    // required to get hot reload to work, but not strictly necessary
    if (!LATTICE_STORE)
      return;

    // update on key change
    let lattice = LATTICE_STORE[props.key];

    // clear elements
    let newElements = [...untrack(elements)];

    // create the correct number of elements
    for (let i = 0; i < lattice.count; i++) {
      let newElement: HTMLCanvasElement;

      if (!newElements[i]) {
        newElement = canvasElement(i, intersectionObserver);
        newElements.push(newElement);
      } else {
        newElement = newElements[i];
      }

      let [width, height] = lattice.getDimensions(i);

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

    // add elements

    for (let i = prev.count; i < props.count; i++) {
      // returns true if new element is created
      if ((i + 1) / latticeList.maxPerLattice > elements().length) {
        setElements([...elements(), canvasElement(elements().length, intersectionObserver)]);
      }

      // check to see if we need to resize the lattice
      let lattice = latticeList.store[elements().length - 1];
      let [width, height] = lattice.get_dimensions();
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
      "random-walk": new LatticeList(() => new RandomWalkLattice(8, randomSeed()), 1, 8),
      "random-teleport": new LatticeList(() => new RandomTeleportLattice(5, randomSeed()), 1, 5),
      "hold-left": new LatticeList(() => new HoldLeftLattice(4, randomSeed()), 1, 4),
      "tremaux": new LatticeList(() => new TremauxLattice(3, randomSeed()), 1, 3),
      "time-travel": new LatticeList(() => new TimeTravelLattice(3, randomSeed()), 1, 3),
      "clone": new LatticeList(() => new CloneLattice(2, randomSeed()), 1, 2),
    };

    // add initial count to each elements
    for (let [key, lattice] of Object.entries(LATTICE_STORE)) {
      let count = shop.find(x => x.key == key).count;

      for (let i = 0; i < count; i++) {
        // this is blocking which is kinda bad
        lattice.push();
      }
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
  const [shownMazeType, setShownMazeType] = createSignal<ShopKey>("random-walk");

  const shownMazeItem = () => shop.find(el => el.key == shownMazeType());
  const [latticeScale, setLatticeScale] = createSignal(1);

  const [fullscreen, setFullscreen] = createSignal(false);

  return (
    <div class="w-full flex flex-col" ref={mazeDisplay}>
      <div class="p-8 bg-black text-white font-pixelated flex">
        <select class="bg-black text-xl hover:bg-white hover:text-black transition-colors" onChange={(e) => {
          setLatticeScale(1);
          setShownMazeType(e.currentTarget.value as ShopKey);
        }}>
          <For each={shop}>
            {item => <option value={item.key} class="py-4 bg-white text-black">{SHOP[item.key].name}</option>}
          </For>
        </select>

        <div class="text-center ml-auto flex">
          <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeScale(x => Math.max(x - 1, 1))}>-</button>
          <p class="bg-white text-black p-2">{latticeScale()}</p>
          <button class="hover:bg-white hover:text-black transition-all p-2 select-none" onClick={() => setLatticeScale(x => Math.min(x + 1, 3))}>+</button>

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
          <SnailLatticeElement key={shownMazeItem().key} count={shownMazeItem().count} scale={latticeScale()} />
        </div>
      }
    </div>
  )
};

export default AutoMazes;
