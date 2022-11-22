// each cell is stores which walls it has in the lower 4 bits:
// 0  0    0    0
// up down left right
onmessage = function(e: MessageEvent<{ width: number, height: number }>) {
  let { width, height } = e.data;

  let grid = new Uint8Array(width * height).fill(15);
  let visited = Array(width * height).fill(false);

  const coords = (x: number, y: number): number => {
    return y * width + x;
  }

  const randomOrder = (): number[] => {
    let elements = [0, 1, 2, 3];
    for (let i = elements.length - 1; i > 0; i--) {
      let j = Math.floor(Math.random() * (i + 1));
      let temp = elements[i];
      elements[i] = elements[j];
      elements[j] = temp;
    }

    return elements;
  }

  const randomWalk = (x: number, y: number) => {
    visited[coords(x, y)] = true;

    for (let direction of randomOrder()) {
      // right
      if (direction == 0 && x + 1 < width && !visited[coords(x + 1, y)]) {
        grid[coords(x, y)] ^= 1;
        grid[coords(x + 1, y)] ^= 2;
        randomWalk(x + 1, y)
        break;
      }

      // left
      if (direction == 1 && x - 1 >= 0 && !visited[coords(x - 1, y)]) {
        grid[coords(x, y)] ^= 2;
        grid[coords(x - 1, y)] ^= 1;
        randomWalk(x - 1, y)
        break;
      }

      // up
      if (direction == 2 && y - 1 >= 0 && !visited[coords(x, y - 1)]) {
        grid[coords(x, y)] ^= 8;
        grid[coords(x, y - 1)] ^= 4;
        randomWalk(x, y - 1)
        break;
      }

      // down
      if (direction == 3 && y + 1 < height && !visited[coords(x, y + 1)]) {
        grid[coords(x, y)] ^= 4;
        grid[coords(x, y + 1)] ^= 8;
        randomWalk(x, y + 1)
        break;
      }
    }
  };

  randomWalk(0, 0);

  for (let x = 0; x < width; x++) {
    for (let y = 0; y < height; y++) {
      if (!visited[coords(x, y)]) {
        for (let direction of randomOrder()) {
          if (direction == 0 && x + 1 < width && visited[coords(x + 1, y)]) {
            grid[coords(x, y)] ^= 1;
            grid[coords(x + 1, y)] ^= 2;
            randomWalk(x, y);
            break;
          }

          if (direction == 1 && x - 1 >= 0 && visited[coords(x - 1, y)]) {
            grid[coords(x, y)] ^= 2;
            grid[coords(x - 1, y)] ^= 1;
            randomWalk(x, y);
            break;
          }

          if (direction == 2 && y - 1 >= 0 && visited[coords(x, y - 1)]) {
            grid[coords(x, y)] ^= 8;
            grid[coords(x, y - 1)] ^= 4;
            randomWalk(x, y);
            break;
          }

          if (direction == 3 && y + 1 < height && visited[coords(x, y + 1)]) {
            grid[coords(x, y)] ^= 4;
            grid[coords(x, y + 1)] ^= 8;
            randomWalk(x, y);
            break;
          }
        }
      }
    }
  }

  postMessage(grid);
}

export type {};
