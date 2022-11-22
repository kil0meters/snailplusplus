import { Component, createEffect, createSignal, onMount, untrack } from "solid-js";
import snail from '../assets/snail.png';
import goal from '../assets/goal.png';
import { generateMaze } from './utils';

export interface BaseMazeProps {
  onScore: (score: number) => void;
  width: number;
  class?: string;
  height: number;
  animate?: boolean;
};

interface SnailMazeProps extends BaseMazeProps {
  movement: number;
  onMove?: (cell: number) => void;
};

export const SNAIL_MOVEMENT_TIME = 250;

const SnailMaze: Component<SnailMazeProps> = (props) => {
  const [grid, setGrid] = createSignal(new Uint8Array);
  let canvas: HTMLCanvasElement | undefined;

  let draw: () => void;
  let onMove: () => boolean | null;
  let isVisible: boolean = true;

  // logical player position, origin at top left
  const [position, setPosition] = createSignal([0, 0]);

  // tracks the animation percentage of the player movement
  const [movementProgress, setMovementProgress] = createSignal([0.0, 0.0]);

  const [loaded, setLoaded] = createSignal(false);

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

  let inputQueue: number[] = [];
  let isMoving = false;

  const animateSnailMoving = (): boolean => {
    let movement = inputQueue.shift();

    let currentPosition = position();
    let targetPosition = nextPosition(movement);


    if (currentPosition[0] === targetPosition[0]
      && currentPosition[1] === targetPosition[1]) {
      isMoving = false;

      if (inputQueue.length == 0) return false;
      else return animateSnailMoving();
    };

    setLastMovement(movement);

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
        setPosition(targetPosition);
        setMovementProgress([0, 0]);
        isMoving = false;
        if (onMove) onMove();
        if (inputQueue.length != 0) animateSnailMoving();
      }

      draw();
    }

    requestAnimationFrame(animateInner);

    return true;
  }

  const moveSnail = (): boolean => {
    if (isMoving) return true;
    isMoving = true;

    if (props.animate && isVisible) {
      return animateSnailMoving();
    }

    let movement = inputQueue.shift();
    setLastMovement(movement);
    setPosition(nextPosition(movement));

    setTimeout(() => {
      isMoving = false;
      if (onMove) onMove();
      draw();
    }, SNAIL_MOVEMENT_TIME);

    return true;
  };

  const [lastMovement, setLastMovement] = createSignal(1);

  // handle movement signal
  createEffect(() => {
    if (!loaded()) return;

    let movement = props.movement;

    if (movement != 0) {
      onMove = () => {
        inputQueue.push(props.movement);
        return moveSnail();
      };

      untrack(() => {
        if (!isMoving && !onMove()) {
          onMove = null;
        }
      });
    } else if (onMove && movement == 0) {
      onMove = null;
    }
  }, [props.movement]);

  createEffect(() => {
    if (props.onMove) {
      props.onMove(
        grid()[position()[1] * props.width + position()[0]]
      );
    }

    // on win
    if (position()[0] == props.width - 1 && position()[1] == props.height - 1) {
      setPosition([0, 0]);
      generateMaze(props.width, props.height, (maze) => {
        setGrid(maze);
      });
      props.onScore(props.width * props.height);
    }
  }, [position]);

  onMount(() => {
    generateMaze(props.width, props.height, (maze) => {
      setGrid(maze);
      setLoaded(true);
    });

    if (!canvas) return

    let ctx = canvas.getContext("2d", { alpha: false });
    ctx.fillStyle = "#110aef";
    ctx.imageSmoothingEnabled = false;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    let snailImage = new Image;
    snailImage.src = snail;

    let goalImage = new Image;
    goalImage.src = goal;

    snailImage.onload = () => {
      requestAnimationFrame(draw);
    }

    goalImage.onload = () => {
      requestAnimationFrame(draw);
    }

    let gridCanvas = document.createElement('canvas');
    gridCanvas.width = canvas.width;
    gridCanvas.height = canvas.height;

    let canvasWorker = gridCanvas.transferControlToOffscreen();

    let renderGridWorker = new Worker(new URL("./drawMaze.ts", import.meta.url));
    renderGridWorker.postMessage(
      { canvas: canvasWorker },
      [canvasWorker]
    );

    // render grid whenever grid changes
    createEffect(() => {
      renderGridWorker.postMessage({
        grid: grid(),
        width: props.width,
        height: props.height
      })
    });

    let animation = false;
    setInterval(() => {
      animation = !animation;
      draw();
    }, 500);

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

    draw = () => {
      if (!isVisible) return;

      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.drawImage(gridCanvas, 0, 0);

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
    };
  });

  let container: HTMLDivElement | undefined;
  const [scale, setScale] = createSignal(1);

  const updateScale = () => {
    const scaleX = container.clientWidth / canvas.width;
    const scaleY = container.clientHeight / canvas.height;
    setScale(Math.min(scaleX, scaleY));
  }

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
