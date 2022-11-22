import { Component, createEffect, createSignal } from 'solid-js';
import SnailMaze, { BaseMazeProps, SNAIL_MOVEMENT_TIME } from '../SnailMaze';

interface HoldLeftProps extends BaseMazeProps {}

function rotateClockwise(movement: number): number {
  switch (movement) {
    // right
    case 1: return 4;
    // left
    case 2: return 8;
    // down
    case 4: return 2;
    // up
    case 8: return 1;
  }

  return 0;
}

function rotateCounterClockwise(movement: number): number {
  switch (movement) {
    // right
    case 1: return 8;
    // left
    case 2: return 4;
    // down
    case 4: return 1;
    // up
    case 8: return 2;
  }

  return 0;
}

const HoldLeftMaze: Component<HoldLeftProps> = (props) => {
  const onMove = (front: number, cell: number, callback: (next: number) => void) => {
    let left = rotateCounterClockwise(front);

    // if no left wall, make left turn
    if((cell & left) == 0) {
      front = left;
    }

    // otherwise, if there is also a wall blocking the front, rotate right
    else if ((cell & front) != 0) {
      front = rotateClockwise(front);
    }

    callback(front);
  };

  return (
    <SnailMaze
      animate={true}
      onMove={onMove}
      height={props.height}
      width={props.width}
      onScore={props.onScore}
      class={props.class}
    />
  );
}

export default HoldLeftMaze;
