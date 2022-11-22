import { Component, createSignal } from 'solid-js';
import SnailMaze, { BaseMazeProps } from '../SnailMaze';

const RandomWalkMaze: Component<BaseMazeProps> = (props) => {
  let [movement, setMovement] = createSignal(0);

  setInterval(() => {
    setMovement(1 << Math.floor(Math.random() * 4));
  }, 250)

  return (
    <SnailMaze
      movement={movement()}
      height={props.height}
      width={props.width}
      onScore={props.onScore}
      class={props.class}
    />
  );
}

export default RandomWalkMaze;
