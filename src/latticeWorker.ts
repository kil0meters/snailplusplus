import init, { WasmLattice } from "../snail-lattice/pkg/snail_lattice";
import type { ShopKey } from "./ShopProvider";
import { randomSeed } from "./utils";

// This class stores an array of SnailLattices, and manages the web worker
// threads therein. Using intersection observers we can only render mazes that
// are in view, dynamically creating and removing canvases based on the
// viewport.
class LatticeList {
    lattice: WasmLattice;
    mazeType: ShopKey;
    width: number;
    prevTick: number;
    tickRate: number = 1;
    score: number;

    get latticeCount(): number {
        return Math.ceil(this.lattice.count() / this.pageSize);
    }

    get pageSize(): number {
        return this.width;
    }

    constructor(mazeType: ShopKey, lattice: WasmLattice) {
        this.mazeType = mazeType;
        this.lattice = lattice;
        this.width = lattice.get_width();
        this.prevTick = performance.now();
    }

    setTickRate(rate: number) {
        this.tickRate = rate;
    }

    getDimensions(): Uint32Array {
        return this.lattice.get_dimensions(this.pageSize);
    }

    getSolveCount(): Uint32Array {
        return this.lattice.get_solve_count();
    }

    setUpgrades(upgrades: number) {
        this.lattice.set_upgrades(upgrades);
    }

    // tick everything
    tick(): number {
        let now = performance.now();
        let dt = now - this.prevTick;
        this.prevTick = performance.now();

        return this.lattice.tick(dt * this.tickRate);
    }

    render(pages: { page: number, buffer: Uint8ClampedArray }[]) {
        let buffers = [];

        for (let page of pages) {
            // @ts-ignore -- wasm-bindgen limitation, can't specify uint8clamped array
            // in the type signature easily
            this.lattice.render(page.buffer, page.page * this.pageSize, this.pageSize);

            buffers.push(page.buffer.buffer);
        }

        postMessage({ type: "render", pages, mazeType: this.mazeType }, buffers);
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
    | { type: "set-tick-rate", rate: number }
    | { type: "render", pages: { page: number, buffer: Uint8ClampedArray }[] }
    | { type: "reset" }
    | { type: "set-upgrades", upgrades: number }
    | { type: "alter", diff: number }
    | { type: "get-count" };

export type LatticeWorkerResponse =
    | { type: "score", score: number, solves: Uint32Array, mazeType: ShopKey }
    | { type: "render", pages: { page: number, buffer: Uint8ClampedArray }[], mazeType: ShopKey }
    | { type: "lattice-updated", width: number, height: number, latticeCount: number };

let LATTICE: LatticeList;

function setupLattice(mazeType: ShopKey) {
    init().then(() => {
        LATTICE = new LatticeList(mazeType, new WasmLattice(mazeType, randomSeed()));

        setInterval(() => {
            let score = LATTICE.tick() + LATTICE.score;
            let solves = LATTICE.getSolveCount();
            LATTICE.score = 0;

            if (score > 0) {
                postMessage({
                    type: "score",
                    score,
                    solves,
                    mazeType,
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
        case "set-tick-rate":
            if (!LATTICE) messageQueue.push(msg);
            else LATTICE.setTickRate(msg.rate);
            break;
        case "set-upgrades":
            if (!LATTICE) messageQueue.push(msg);
            else LATTICE.setUpgrades(msg.upgrades);
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
