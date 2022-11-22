import { Component, createEffect, createSignal } from 'solid-js';
import SnailMaze, { BaseMazeProps, SNAIL_MOVEMENT_TIME } from '../SnailMaze';

interface RandomWalkProps extends BaseMazeProps {
  glasses?: boolean,
}

const RandomWalkMaze: Component<RandomWalkProps> = (props) => {
  let timeoutId: number | undefined;

  const onMove = (_movement: number, cell: number, callback: (next: number) => void) => {
    if (timeoutId) clearTimeout(timeoutId);

    if (props.glasses) {
      let options = [];

      if ((cell & 1) == 0) options.push(1);
      if ((cell & 2) == 0) options.push(2);
      if ((cell & 4) == 0) options.push(4);
      if ((cell & 8) == 0) options.push(8);

      callback(options[Math.floor(Math.random() * options.length)]);
    } else {
      let movement = 1 << Math.floor(Math.random() * 4);

      if ((cell & movement) != 0) {
        timeoutId = setTimeout(callback, SNAIL_MOVEMENT_TIME, movement);
      } else {
        callback(movement);
      }
    }
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

export default RandomWalkMaze;
