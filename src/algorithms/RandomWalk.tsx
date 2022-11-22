import { Component, createEffect, createSignal } from 'solid-js';
import SnailMaze, { BaseMazeProps } from '../SnailMaze';

interface RandomWalkProps extends BaseMazeProps {
  glasses?: boolean,
}

const RandomWalkMaze: Component<RandomWalkProps> = (props) => {
  let [movement, setMovement] = createSignal(0);

  const onMove = (cell: number) => {
    if (props.glasses) {
      let options = [];

      if ((cell & 1) == 0) options.push(1);
      if ((cell & 2) == 0) options.push(2);
      if ((cell & 4) == 0) options.push(4);
      if ((cell & 8) == 0) options.push(8);

      setMovement(options[Math.floor(Math.random() * options.length)]);
    } else {
      setMovement(1 << Math.floor(Math.random() * 4));
    }
  };

  return (
    <SnailMaze
      animate={true}
      movement={movement()}
      onMove={onMove}
      height={props.height}
      width={props.width}
      onScore={props.onScore}
      class={props.class}
    />
  );
}

export default RandomWalkMaze;
