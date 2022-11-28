import { Component, createEffect, createSignal, onCleanup, useContext, For } from 'solid-js';
import music from '../assets/gameplay.mp3';
import ScoreProvider, { ScoreContext } from './ScoreProvider';
import ShopProvider, { ShopContext, ShopItem } from './ShopProvider';
import "../assets/font.woff2";
import UpgradesProvider, { Upgrade, UpgradesContext } from './UpgradesProvider';
import { createStoredSignal } from './utils';
import AutoMazes from './AutoMazes';
import SnailMaze from './SnailMaze';
import Shop from './Shop';

const Game: Component = () => {
  const [score, setScore] = useContext(ScoreContext);
  const updateScore = (newScore: number) => setScore(score() + newScore);
  const [mazeSize, setMazeSize] = createStoredSignal("maze-size", 5);

  const [displayedScore, setDisplayedScore] = createSignal(score());

  createEffect(() => {
    let difference = score() - displayedScore();
    let prev = new Date();

    if (difference < 0) {
      setDisplayedScore(score());
      return;
    }

    const animate = () => {
      let now = new Date();
      let dt = now.valueOf() - prev.valueOf();
      setDisplayedScore(Math.min(displayedScore() + difference * dt / 1000, score()));

      if (displayedScore() != score()) {
        requestAnimationFrame(animate);
      }
    };

    requestAnimationFrame(animate);
  });

  return (
    <div class='grid grid-cols-[minmax(0,5fr)_minmax(0,3fr)] overflow-hidden bg-[#068fef]'>
      <div class='flex flex-col gap-8 h-full overflow-auto pb-16'>
        <div class='p-8 bg-black flex justify-center'>
          <span class='text-4xl text-center font-extrabold font-pixelated text-white'>{Math.floor(displayedScore())} fragments</span>
        </div>
        <SnailMaze class='min-h-[70vh] h-full' height={mazeSize()} width={mazeSize()} onScore={(score) => { updateScore(score) }} />
        <AutoMazes />
      </div>
      <Shop />
    </div>
  );
};

const App: Component = () => {
  let audio;
  const [gameStarted, setGameStarted] = createSignal(true);

  const startGame = () => {
    setGameStarted(true);

    audio = new Audio(music);
    audio.onended = () => {
      audio.play();
    }
    audio.play()
  }

  onCleanup(() => {
    if (audio) {
      audio.onended = undefined;
      audio.pause();
    }
  });

  return (
    <UpgradesProvider>
      <ShopProvider>
        <ScoreProvider>
          <div class='h-screen grid'>
            {gameStarted() ? (
              <Game />
            ) : (
              <div class='flex flex-col gap-8 w-96 self-center justify-self-center text-center'>
                <h1 class='text-5xl font-extrabold'>Snail Maze</h1>
                <button onClick={startGame} class='border-4 font-extrabold text-3xl py-4 px-8 border-black hover:bg-black hover:text-white transition-colors'>Play</button>
              </div>
            )}
          </div>
        </ScoreProvider>
      </ShopProvider>
    </UpgradesProvider>
  );
};

export default App;
