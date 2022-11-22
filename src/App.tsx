import { Component, createSignal, For, onCleanup, useContext } from 'solid-js';
import music from '../assets/gameplay.mp3';
import PlayerMaze from './algorithms/Player';
import RandomWalkMaze from './algorithms/RandomWalk';
import ScoreProvider, { ScoreContext } from './ScoreProvider';
import ShopProvider, { ShopContext, ShopItem } from './ShopProvider';


const PRICE_SCALER = 1.15;

const ShopListing: Component<ShopItem> = (props) => {
  const [score, setScore] = useContext(ScoreContext);
  const [_shop, setShop] = useContext(ShopContext);

  const price = () => Math.floor(props.price * Math.pow(PRICE_SCALER, props.count));

  const buy = () => {
    if (score() >= price()) {
      setScore(score() - price())
      setShop(
        (shopItem) => shopItem.key === props.key,
        "count",
        (count) => count + 1
      );
    }
  };

  return (
    <button onClick={buy} class='flex hover:bg-neutral-100 p-4 transition-colors text-left'>
      <div class='flex flex-col'>
        <span class='text-2xl font-extrabold'>{ props.name }</span>
        <span class=''>{price()} MF</span>
      </div>

      {props.count > 0 && <span class='ml-auto font-extrabold text-3xl self-center'>{props.count}</span>}
    </button>
  );
}

const Shop: Component = () => {
  const [shop, setShop] = useContext(ShopContext);

  const reset = () => {
    setShop(() => true, "count", () => 0);
  };

  return (
    <div class="bg-white overflow-hidden flex flex-col shadow-lg border-l-4 border-black">
      <div class='border-b-4 border-black p-4'>
        <h1 class='font-extrabold text-2xl mb-4'>Upgrades</h1>

        <div class='flex gap-4'>
          <button class='aspect-square border-4 border-black p-2 hover:bg-black transition-colors text-black hover:text-white'>Glasses</button>
        </div>
      </div>

      <For each={shop}>{item => <ShopListing
        key={item.key}
        name={item.name}
        description={item.description}
        price={item.price}
        count={item.count}
      />}</For>

      <button onClick={reset} class="bg-red-700 p-4 hover text-red-50 hover:bg-red-600 transition-colors">
        Reset
      </button>
    </div>
  );
}

const ShopMazes: Component<ShopItem> = (props) => {
  const [score, setScore] = useContext(ScoreContext);

  const updateScore = (newScore: number) => setScore(score() + newScore);

  if (props.key == "random-walk") {
    return (
      <For each={Array(props.count)}>{() =>
        <RandomWalkMaze class='w-[101px] h-[101px]' height={5} width={5} onScore={updateScore} />
      }</For>
    );
  }

  return <></>;
};

const AutoMazes: Component = () => {
  const [shop, _setShop] = useContext(ShopContext);

  return (
    <div class="self-center grid grid-cols-[repeat(7,min-content)]">
      <For each={shop}>
        {item => <ShopMazes
          key={item.key}
          name={item.name}
          description={item.description}
          price={item.price}
          count={item.count}
          />}
      </For>
    </div>
  )
};

const Game: Component = () => {
  const [score, setScore] = useContext(ScoreContext);
  const updateScore = (newScore: number) => setScore(score() + newScore);

  return (
    <div class='grid grid-cols-[minmax(0,5fr)_minmax(0,3fr)] overflow-hidden'>
      <div class='flex flex-col gap-8 h-full overflow-auto p-8'>
        <span class='text-4xl text-center font-extrabold'>{score()} MAZE FRAGMENTS</span>
        <PlayerMaze class='min-h-[70vh] h-full' height={10} width={10} onScore={updateScore} />
        <AutoMazes />
      </div>
      <Shop />
    </div>
  );
};

const App: Component = () => {
  let audio;
  const [gameStarted, setGameStarted] = createSignal(false);

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
  );
};

export default App;
