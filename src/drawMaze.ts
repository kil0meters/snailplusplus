export default function drawMaze(canvas, grid, width, height) {
  canvas.width = width * 10 + 1;
  canvas.height = height * 10 + 1;

  console.log(`Generating ${width}x${height} maze`);

  let ctx: CanvasRenderingContext2D  = canvas.getContext("2d", { alpha: false });

  ctx.resetTransform();
  ctx.translate(.5,.5);
  ctx.lineWidth = 1;
  ctx.fillStyle = "#110aef";
  ctx.strokeStyle = "#068fef";

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
