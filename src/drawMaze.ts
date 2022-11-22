// TODO: Switch to uint8array
let ctx: OffscreenCanvasRenderingContext2D;
let canvas: OffscreenCanvas;

type RenderGrid = {
  grid: Uint8Array,
  width: number,
  height: number,
};

type SetCanvas = {
 canvas: OffscreenCanvas
};

onmessage = function(e: MessageEvent<RenderGrid | SetCanvas>) {
  if ("canvas" in e.data) {
    canvas = e.data.canvas;

    //@ts-ignore
    ctx = canvas.getContext("2d", { alpha: false });

    ctx.translate(.5,.5);
    ctx.lineWidth = 1;
    ctx.fillStyle = "#110aef";
    ctx.strokeStyle = "#068fef";

    return;
  }

  let {grid, width, height } = e.data;

  // ctx.clearRect(0, 0, canvas.width, gridCanvas.height);
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.beginPath();

  for (let x = 0; x < width; x++) {
    for (let y = 0; y < height; y++) {
      let cell = grid[y * width + x];
      // right
      if ((cell & 1) != 0) {
        ctx.moveTo((x + 1) * 10, y * 10);
        ctx.lineTo((x + 1) * 10, (y + 1) * 10);
        ctx.stroke();
      }

      // left
      if ((cell & 2) != 0) {
        ctx.moveTo(x * 10, y * 10);
        ctx.lineTo(x * 10, (y + 1) * 10);
        ctx.stroke();
      }

      // down
      if ((cell & 4) != 0) {
        ctx.moveTo(x * 10, (y + 1) * 10);
        ctx.lineTo((x + 1) * 10, (y + 1) * 10);
        ctx.stroke();
      }

      // up
      if ((cell & 8) != 0) {
        ctx.moveTo(x * 10, y * 10);
        ctx.lineTo((x + 1) * 10, y * 10);
        ctx.stroke();
      }
    }
  }
}

export type {}
