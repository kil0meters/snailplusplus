import { Component, createSignal, onCleanup } from 'solid-js';
import music from '../assets/gameplay.mp3';
import ScoreProvider from './ScoreProvider';
import ShopProvider, { ShopKey } from './ShopProvider';
import "../assets/font.woff2";
import UpgradesProvider from './UpgradesProvider';
import Game from './Game';

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
