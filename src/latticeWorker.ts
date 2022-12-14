import init, { CloneLattice, HoldLeftLattice, RandomTeleportLattice, RandomWalkLattice, TimeTravelLattice, TremauxLattice, LearningLattice } from "../snail-lattice/pkg/snail_lattice";
import { ShopKey } from "./ShopProvider";

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
  score: number;

  get latticeCount(): number {
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

  // tick everything
  tick(): number {
    let now = performance.now();
    let dt = Math.floor((now - this.prevTick) * 1000);
    this.prevTick = performance.now();

    return this.lattice.tick(dt);
  }

  render(pages: { page: number, buffer: Uint8ClampedArray }[]) {
    let buffers = [];

    for (let page of pages) {
      // @ts-ignore -- wasm-bindgen limitation, can't specify uint8clamped array
      // in the type signature easily
      this.lattice.render(page.buffer, page.page * this.pageSize, this.pageSize);

      buffers.push(page.buffer.buffer);
    }

    postMessage({ type: "render", pages }, buffers);
  }

  // fucks up if it doesn't divide evenly right now
  setWidth(width: number) {
    this.width = width;
    this.lattice.set_width(this.width);
  }

  update() {
    let [width, height] = this.getDimensions();

    postMessage({
      type: "lattice-updated",
      width,
      height,
      latticeCount: LATTICE.latticeCount
    });
  }

  alter(diff: number) {
    let prev = this.latticeCount;

    this.lattice.alter(diff);

    let latticeDiff = this.latticeCount - prev;

    if (latticeDiff > 0) {
      this.update();
    }
  }
};

export type LatticeWorkerMessage =
  | { type: "setup", mazeType: ShopKey }
  | { type: "set-width", width: number }
  | { type: "render", pages: { page: number, buffer: Uint8ClampedArray }[] }
  | { type: "reset" }
  | { type: "alter", diff: number }
  | { type: "get-count" }

export type LatticeWorkerResponse =
  | { type: "score", score: number }
  | { type: "render", pages: { page: number, buffer: Uint8ClampedArray }[] }
  | { type: "lattice-updated", width: number, height: number, latticeCount: number }

let LATTICE: LatticeList<SnailLattice>;

function randomSeed(): number {
  return self.crypto.getRandomValues(new Uint16Array(1))[0];
}

function setupLattice(mazeType: ShopKey) {
  init().then(() => {
    switch (mazeType) {
      case "random-walk":
        LATTICE = new LatticeList(new RandomWalkLattice(8, randomSeed()), 1, 8);
        break;
      case "random-teleport":
        LATTICE = new LatticeList(new RandomTeleportLattice(5, randomSeed()), 1, 5);
        break;
      case "learning":
        LATTICE = new LatticeList(new LearningLattice(3, randomSeed()), 1, 3);
        break;
      case "hold-left":
        LATTICE = new LatticeList(new HoldLeftLattice(4, randomSeed()), 1, 4);
        break;
      case "tremaux":
        LATTICE = new LatticeList(new TremauxLattice(3, randomSeed()), 1, 3);
        break;
      case "time-travel":
        LATTICE = new LatticeList(new TimeTravelLattice(3, randomSeed()), 1, 3);
        break;
      case "clone":
        LATTICE = new LatticeList(new CloneLattice(2, randomSeed()), 1, 2);
        break;
    }

    setInterval(() => {
      let score = LATTICE.tick() + LATTICE.score;
      LATTICE.score = 0;

      if (score > 0) {
        if (LATTICE.width == 12)
          console.log(score);
        postMessage({
          type: "score",
          score: score,
        })
      }
    }, 100)

    let msg = messageQueue.pop();

    while (msg !== undefined) {
      processMessage(msg);
      msg = messageQueue.pop();
    }
  })
}

let messageQueue: LatticeWorkerMessage[] = [];

function processMessage(msg: LatticeWorkerMessage) {
  switch (msg.type) {
    case "setup":
      setupLattice(msg.mazeType);
      break;
    case "reset":
      LATTICE.alter(-LATTICE.lattice.count());
      break;
    case "set-width":
      if (!LATTICE) messageQueue.push(msg);
      else {
        LATTICE.setWidth(msg.width);
        LATTICE.update();
      }
      break;
    case "alter":
      if (!LATTICE) messageQueue.push(msg);
      else LATTICE.alter(msg.diff);
      break;

    // we intentionally don't add this to the message queue because if this is
    // received after the setup went through when it will get the original "add
    // lattice message"
    case "get-count":
      if (LATTICE) LATTICE.update();
      break;
    case "render":
      if (!LATTICE) return;

      LATTICE.score += LATTICE.tick();
      LATTICE.render(msg.pages);

      break;
  }

}

onmessage = (event: MessageEvent<LatticeWorkerMessage>) => processMessage(event.data);
