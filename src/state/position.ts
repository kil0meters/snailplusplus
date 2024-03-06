import { createStoredSignal } from "../util";

// global position
export default createStoredSignal("view-position", { x: 0, y: 0, scale: 1 });
