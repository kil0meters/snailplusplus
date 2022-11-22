import { Component, createEffect, createSignal, onMount } from 'solid-js';
import SnailMaze, { BaseMazeProps } from '../SnailMaze';

const PlayerMaze: Component<BaseMazeProps> = (props) => {
  let [movement, setMovement] = createSignal(0);

  const keyPressed = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
      case 'w':
      case 'W':
        setMovement(8);
        break;
      case 'a':
      case 'A':
        setMovement(2);
        break;
      case 's':
      case 'S':
        setMovement(4);
        break;
      case 'd':
      case 'D':
        setMovement(1);
        break;
    }
  };

  const keyReleased = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
      case 'w':
      case 'W':
        setMovement((movement() == 8) ? 0 : movement);
        break;
      case 'a':
      case 'A':
        setMovement((movement() == 2) ? 0 : movement);
        break;
      case 's':
      case 'S':
        setMovement((movement() == 4) ? 0 : movement);
        break;
      case 'd':
      case 'D':
        setMovement((movement() == 1) ? 0 : movement);
        break;
    }
  };

  let divRef: HTMLDivElement;

  onMount(() => {
    if (divRef) divRef.focus();
  });

  return (
    <div
      ref={divRef}
      class='h-full outline-none'
      tabindex={-1}
      onKeyDown={keyPressed}
      onKeyUp={keyReleased}>

      <SnailMaze
        animate={true}
        movement={movement()}
        height={props.height}
        width={props.width}
        onScore={props.onScore}
        class={props.class}
      />
    </div>
  );

}

export default PlayerMaze;
