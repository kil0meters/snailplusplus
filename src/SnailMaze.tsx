import { Component, createEffect, createSignal, onMount } from "solid-js";
import snail from '../assets/snail.png';
import goal from '../assets/goal.png';
import { generateMaze } from './utils';

export interface BaseMazeProps {
  onScore: (score: number) => void;
  width: number;
  class?: string;
  height: number;
};

interface SnailMazeProps extends BaseMazeProps {
  movement: number;
};

const SnailMaze: Component<SnailMazeProps> = (props) => {
  const [grid, setGrid] = createSignal(new Uint8Array);
  let canvas: HTMLCanvasElement | undefined;

  let draw: () => void;

  // logical player position, origin at top left
  const [position, setPosition] = createSignal([0, 0]);

  // tracks the animation percentage of the player movement
  const [movementProgress, setMovementProgress] = createSignal([0.0, 0.0]);

  const [loaded, setLoaded] = createSignal(false);

  const moveSnail = (currentMovement: number): number[] => {
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

  // animates player moving, 0.1s per move
  const animateSnailMoving = (currentPosition: number[], targetPosition: number[]) => {
    // we only move if the player is not doing another movement.
    // TODO: consider adding an input queue
    if (movementProgress()[0] != 0 || movementProgress()[1] != 0) return;

    let prev = new Date();
    let movementDifference = [targetPosition[0] - currentPosition[0], targetPosition[1] - currentPosition[1]];
    let currentMovementProgress = [0, 0];

    const animateInner = () => {
      let now = new Date();
      let dt = now.valueOf() - prev.valueOf();

      // right
      if (movementDifference[0] > 0) {
        currentMovementProgress[0] = Math.min(currentMovementProgress[0] + movementDifference[0] * dt / 100, movementDifference[0])
      }

      // left
      if (movementDifference[0] < 0) {
        currentMovementProgress[0] = Math.max(currentMovementProgress[0] + movementDifference[0] * dt / 100, movementDifference[0])
      }

      // down
      if (movementDifference[1] > 0) {
        currentMovementProgress[1] = Math.min(currentMovementProgress[1] + movementDifference[1] * dt / 100, movementDifference[1])
      }

      // up
      if (movementDifference[1] < 0) {
        currentMovementProgress[1] = Math.max(currentMovementProgress[1] + movementDifference[1] * dt / 100, movementDifference[1])
      }

      // I don't know why we need to do prop spreading here.
      setMovementProgress([...currentMovementProgress]);

      if (movementDifference[0] != currentMovementProgress[0] || movementDifference[1] != currentMovementProgress[1]) {
        prev = new Date();
        requestAnimationFrame(animateInner);
      } else {
        setPosition(targetPosition);
        setMovementProgress([0, 0]);
      }

      draw();
    }

    requestAnimationFrame(animateInner);
  }

  const [lastMovement, setLastMovement] = createSignal(1);

  let moveIntervalId: number | null;

  // handle movement signal
  createEffect(() => {
    if (!loaded()) return;

    if (props.movement != 0 && !moveIntervalId) {
      animateSnailMoving(position(), moveSnail(props.movement));
      setLastMovement(props.movement);
      moveIntervalId = setInterval(() => {
        animateSnailMoving(position(), moveSnail(props.movement));
        setLastMovement(props.movement);
      }, 150);
    } else if (moveIntervalId && props.movement == 0) {
      clearInterval(moveIntervalId);
      moveIntervalId = null;
    }
  }, [props.movement]);

  // on win
  createEffect(() => {
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

    const ob = new ResizeObserver(() => {
      updateScale();
    });
    ob.observe(container);
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
