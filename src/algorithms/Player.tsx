import { Component, createEffect, createSignal, onMount, untrack } from 'solid-js';
import SnailMaze, { BaseMazeProps } from '../SnailMaze';

const PlayerMaze: Component<BaseMazeProps> = (props) => {
  let inputQueue: number[] = [];
  let cb: (movement: number) => void | undefined;

  function queueEnqueue(queue: number[], add: number) {
    if (queue.length >= 2) queue[1] = add;
    else queue.push(add);

    if (cb && queue.length == 1) {
      cb(add);
      cb = undefined;
    }
  }

  function queueDequeue(queue: number[], remove: number) {
    let removed: number | undefined;

    if (queue[0] === remove) removed = queue.shift();
    if (queue[1] === remove) removed = queue.pop();

    if (cb && removed && queue.length != 0) {
      cb(queue[0]);
      cb = undefined;
    }
  }

  const keyPressed = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
      case 'w':
      case 'W':
        queueEnqueue(inputQueue, 8);
        break;
      case 'a':
      case 'A':
        queueEnqueue(inputQueue, 2);
        break;
      case 's':
      case 'S':
        queueEnqueue(inputQueue, 4);
        break;
      case 'd':
      case 'D':
        queueEnqueue(inputQueue, 1);
        break;
    }
  };

  const keyReleased = (e: KeyboardEvent) => {
    if (e.repeat) return;

    switch (e.key) {
      case 'w':
      case 'W':
        queueDequeue(inputQueue, 8);
        break;
      case 'a':
      case 'A':
        queueDequeue(inputQueue, 2);
        break;
      case 's':
      case 'S':
        queueDequeue(inputQueue, 4);
        break;
      case 'd':
      case 'D':
        queueDequeue(inputQueue, 1);
        break;
    }
  };

  const onMove = (_: number, cell: number, callback: (next: number) => void) => {
    if (inputQueue.length == 0 || (cell & inputQueue[0]) != 0) {
      cb = callback;
    } else {
      callback(inputQueue[0]);
      cb = undefined;
    }
  }

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
        onMove={onMove}
        height={props.height}
        width={props.width}
        onScore={props.onScore}
        class={props.class}
      />
    </div>
  );

}

export default PlayerMaze;
