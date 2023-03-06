import { Component, createContext, createSignal, onCleanup } from 'solid-js';
import music from '../assets/gameplay.mp3';
import ScoreProvider from './ScoreProvider';
import ShopProvider, { ShopKey } from './ShopProvider';
import "../assets/font.woff2";
import UpgradesProvider from './UpgradesProvider';
import Game from './Game';
import { createStore, SetStoreFunction } from 'solid-js/store';
import SnailInfoProvider from './SnailInfoProvider';
import AverageProvider from './AverageProvider';

type Powerup = {
    active: boolean,
    start: Date,
    end: Date,
    multiplier: number,
};

export const PowerupContext = createContext<[Powerup, SetStoreFunction<Powerup>]>();

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

    const [powerup, setPowerup] = createStore<Powerup>({ active: false, start: new Date(), end: new Date(), multiplier: 0 });

    // the ladder of power
    return (
        <PowerupContext.Provider value={[powerup, setPowerup]}>
            <AverageProvider>
                <SnailInfoProvider>
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
                </SnailInfoProvider>
            </AverageProvider>
        </PowerupContext.Provider>
    );
};

export default App;
