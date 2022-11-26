import { batch, Component, createEffect, createMemo, createSignal, on, onCleanup, onMount, untrack } from "solid-js";
import snail from '../assets/snail.png';
import goal from '../assets/goal.png';
import { generateMaze } from './utils';
import init, { SnailLattice } from "snail-lattice";
import drawMaze from "./drawMaze";

export interface BaseMazeProps {
  onScore: (score: number) => void;
  width: number;
  class?: string;
  height: number;
  animate?: boolean;
};

interface SnailMazeProps extends BaseMazeProps {
  onMove: (
    movement: number,
    cell: number,
    callback: (nextMovement: number) => number
  ) => void;
};

export const SNAIL_MOVEMENT_TIME = 250;

const SnailMaze: Component<SnailMazeProps> = (props) => {
  const [grid, setGrid] = createSignal(new Uint8Array);
  let canvas: HTMLCanvasElement | undefined;
  let ctx: CanvasRenderingContext2D;

  let isVisible: boolean = true;

  // logical player position, origin at top left
  const [position, setPosition] = createSignal([0, 0]);

  // tracks the animation percentage of the player movement
  const [movementProgress, setMovementProgress] = createSignal([0.0, 0.0]);

  const nextPosition = (currentMovement: number): number[] => {
    let currentPosition = [...position()];
    let currentCell = grid()[currentPosition[1] * props.width + currentPosition[0]]

    // right
    if ((currentMovement & 1) != 0 && (currentCell & 1) == 0) {
      currentPosition[0] += 1;
    }

    // left
    if ((currentMovement & 2) != 0 && (currentCell & 2) == 0) {
      currentPosition[0] -= 1;
    }

    // down
    if ((currentMovement & 4) != 0 && (currentCell & 4) == 0) {
      currentPosition[1] += 1;
    }

    // up
    if ((currentMovement & 8) != 0 && (currentCell & 8) == 0) {
      currentPosition[1] -= 1;
    }

    return currentPosition;
  };

  let stopped = false;

  // prevents multiple ai tasks being spun up concurrently
  onCleanup(() => {
    stopped = true;
  });

  const movementLoop = () => {
    setTimeout(
      props.onMove,
      0,
      lastMovement(),
      grid()[position()[1] * props.width + position()[0]],
      (next: number) => {
        if (!stopped) moveSnail(next);
      }
    )
  }

  const animateSnailMoving = (movement: number) => {
    let currentPosition = position();
    let targetPosition = nextPosition(movement);

    if (currentPosition[0] === targetPosition[0]
      && currentPosition[1] === targetPosition[1]) {
      movementLoop();
      return;
    };

    let prev = new Date();
    let movementDifference = [targetPosition[0] - currentPosition[0], targetPosition[1] - currentPosition[1]];
    let currentMovementProgress = [0, 0];

    const animateInner = () => {
      let now = new Date();
      let dt = now.valueOf() - prev.valueOf();

      // right
      if (movementDifference[0] > 0) {
        currentMovementProgress[0] = Math.min(currentMovementProgress[0] + movementDifference[0] * dt / SNAIL_MOVEMENT_TIME, movementDifference[0])
      }

      // left
      if (movementDifference[0] < 0) {
        currentMovementProgress[0] = Math.max(currentMovementProgress[0] + movementDifference[0] * dt / SNAIL_MOVEMENT_TIME, movementDifference[0])
      }

      // down
      if (movementDifference[1] > 0) {
        currentMovementProgress[1] = Math.min(currentMovementProgress[1] + movementDifference[1] * dt / SNAIL_MOVEMENT_TIME, movementDifference[1])
      }

      // up
      if (movementDifference[1] < 0) {
        currentMovementProgress[1] = Math.max(currentMovementProgress[1] + movementDifference[1] * dt / SNAIL_MOVEMENT_TIME, movementDifference[1])
      }

      // I don't know why we need to do prop spreading here.
      setMovementProgress([...currentMovementProgress]);

      if (movementDifference[0] != currentMovementProgress[0] || movementDifference[1] != currentMovementProgress[1]) {
        prev = new Date();
        requestAnimationFrame(animateInner);
      } else {
        batch(() => {
          setPosition(targetPosition);
          setMovementProgress([0, 0]);
          movementLoop();
        });
      }

      draw()();
    }

    requestAnimationFrame(animateInner);
  }

  const moveSnail = (movement: number) => {
    setLastMovement(movement);

    if (props.animate && isVisible) {
      animateSnailMoving(movement);
      return;
    }

    setTimeout(() => {
      setPosition(nextPosition(movement));
      draw()();
      movementLoop();
    }, SNAIL_MOVEMENT_TIME);

    return true;
  };

  const [lastMovement, setLastMovement] = createSignal(1);

  createEffect(() => {
    // on win
    if (position()[0] == props.width - 1 && position()[1] == props.height - 1) {
      stopped = true;
      setPosition([0, 0]);
      generateMaze(props.width, props.height, (maze) => {
        setGrid(maze);

        stopped = false;
        movementLoop();
      });
      props.onScore(props.width * props.height);
    }
  });

  createEffect(() => {
    if (!canvas) return;

    canvas.width = props.width * 10 + 1;
    canvas.height = props.height * 10 + 1;

    let ctxi = canvas.getContext("2d", { alpha: false });
    ctxi.fillStyle = "#110aef";
    ctxi.imageSmoothingEnabled = false;
    ctxi.fillRect(0, 0, canvas.width, canvas.height);

    ctx = ctxi;
  });

  const snailImage = new Image;
  snailImage.src = snail;

  const goalImage = new Image;
  goalImage.src = goal;

  // start on mount
  createEffect(() => {
    generateMaze(props.width, props.height, (maze) => {
      setGrid(maze);

      // start movement loop after maze is generated
      stopped = false;
      movementLoop();
    });
  });

  snailImage.onload = () => {
    requestAnimationFrame(draw);
  }

  goalImage.onload = () => {
    requestAnimationFrame(draw);
  }

  const gridCanvas = document.createElement('canvas');

  // render grid whenever grid changes
  createEffect(() => {
    if (grid().length != props.width * props.height) return;

    drawMaze(gridCanvas, grid(), props.width, props.height);
  });

  function drawImage(image: HTMLImageElement, x: number, y: number, rotation: number, flip?: boolean){
    ctx.setTransform(1, 0, 0, 1, Math.floor(x), Math.floor(y)); // sets scale and origin
    if (flip) {
      ctx.scale(-1, 1);
    }
    ctx.rotate(rotation);
    if (animation) {
      ctx.drawImage(image, 8, 0, 8, 8, -4, -4, 8, 8);
    } else {
      ctx.drawImage(image, 0, 0, 8, 8, -4, -4, 8, 8);
    }
  }

  const draw = createMemo(() => () => {
    if (!isVisible) return;

    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.drawImage(gridCanvas, 0, 0);

    // let data = ctx.getImageData(0, 0, canvas.width, canvas.height);

    // // this works (trust me)
    // make_white(data.data as unknown as Uint8Array);
    // ctx.putImageData(data, 0, 0);

    // right
    if ((lastMovement() & 1) != 0) {
      drawImage(
        snailImage,
        (position()[0] + movementProgress()[0]) * 10 + 6,
        (position()[1] + movementProgress()[1]) * 10 + 6,
        0
      );
    }

    // left
    else if ((lastMovement() & 2) != 0) {
      drawImage(
        snailImage,
        (position()[0] + movementProgress()[0]) * 10 + 5,
        (position()[1] + movementProgress()[1]) * 10 + 6,
        0,
        true
      );
    }

    // down
    else if ((lastMovement() & 4) != 0) {
      drawImage(
        snailImage,
        (position()[0] + movementProgress()[0]) * 10 + 5,
        (position()[1] + movementProgress()[1]) * 10 + 6,
        Math.PI/2
      );
    }

    // up
    else if ((lastMovement() & 8) != 0) {
      drawImage(
        snailImage,
        (position()[0] + movementProgress()[0]) * 10 + 6,
        (position()[1] + movementProgress()[1]) * 10 + 5,
        3*Math.PI/2
      );
    }

    drawImage(
      goalImage,
      props.width * 10 - 4,
      props.height * 10 - 4,
      0
    );
  });

  // main animation loop
  let animation = false;
  setInterval(() => {
    animation = !animation;
    draw()();
  }, 500);

  let container: HTMLDivElement | undefined;
  const [scale, setScale] = createSignal(1);

  const updateScale = () => {
    const scaleX = container.clientWidth / canvas.width;
    const scaleY = container.clientHeight / canvas.height;
    setScale(Math.floor(Math.min(scaleX, scaleY)));
  }

  createEffect(() => {
    props.height;
    props.width;

    updateScale();
  });

  onMount(() => {
    if (!container || !canvas) return;
    updateScale();

    const resizeObserver = new ResizeObserver(() => {
      updateScale();
    });

    resizeObserver.observe(container);

    const intersectionObserver = new IntersectionObserver(entries => {
      isVisible = entries[0].isIntersecting;
    }, { threshold: [0] });

    intersectionObserver.observe(container);
  });

  return (
    <div ref={container} class={`flex items-center justify-center ${props.class}`}>
      <canvas
        ref={canvas}
        width={props.width * 10 + 1}
        height={props.height * 10 + 1}
        style={{
          "image-rendering": "pixelated",
          transform: `scale(${scale()})`
        }}
      >
      </canvas>
    </div>
  );
};

export default SnailMaze;
