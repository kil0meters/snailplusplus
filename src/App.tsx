import { Component, createEffect, createSignal, onCleanup, useContext, For } from 'solid-js';
import music from '../assets/gameplay.mp3';
import PlayerMaze from './algorithms/Player';
import RandomWalkMaze from './algorithms/RandomWalk';
import ScoreProvider, { ScoreContext } from './ScoreProvider';
import ShopProvider, { ShopContext, ShopItem } from './ShopProvider';
import "../assets/font.woff2";
import UpgradesProvider, { Upgrade, UpgradesContext } from './UpgradesProvider';
import HoldLeftMaze from './algorithms/HoldLeft';
import { createStoredSignal } from './utils';
import init, { SnailLattice } from "snail-lattice";

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
        (count) => count + 100
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

const UpgradeListing: Component<Upgrade> = (props) => {
  const [score, setScore] = useContext(ScoreContext);
  const [_shop, setUpgrades] = useContext(UpgradesContext);

  const buy = () => {
    if (props.owned) {
      setScore(score() + props.price)
      setUpgrades(
        (item) => item.key === props.key,
        "owned",
        () => false
      );
    } else {
      if (score() >= props.price) {
        setScore(score() - props.price)
        setUpgrades(
          (item) => item.key === props.key,
          "owned",
          () => true
        );
      }
    }
  };

  return <>
    <button
      onClick={buy}
      class={
        `aspect-square border-4 border-black p-2 transition-all outline-black outline outline-0 hover:outline-4 ${props.owned ? "bg-black text-white" : "bg-white text-black"}`
      }>
      {props.name}
    </button>
  </>
}

const Shop: Component = () => {
  const [shop, setShop] = useContext(ShopContext);
  const [upgrades, _setUpgrades] = useContext(UpgradesContext);

  const reset = () => {
    setShop(() => true, "count", () => 0);
  };

  return (
    <div class="bg-white overflow-hidden flex flex-col shadow-lg border-l-4 border-black">
      <div class='border-b-4 border-black p-4'>
        <h1 class='font-extrabold text-2xl mb-4'>Upgrades</h1>

        <div class='flex gap-4'>
          <For each={upgrades}>{item =>
            <UpgradeListing
              key={item.key}
              name={item.name}
              description={item.description}
              price={item.price}
              owned={item.owned}
            />
          }</For>
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
  const [upgrades, _] = useContext(UpgradesContext);

  const updateScore = (newScore: number) => setScore(score() + newScore);

  const shopMazeClasses = 'w-[101px] h-[101px] hover:scale-[200%] hover:z-50 transition-all hover:shadow-blue'

  if (props.key == "random-walk") {
    // let hasGlasses = false; // upgrades.find(x => x.key == "glasses").owned;

    // createEffect(() => {
    //   console.log("hello");
    //   hasGlasses = upgrades.find(x => x.key == "glasses").owned;
    // });

    return (
      <For each={Array(props.count)}>{() =>
        <RandomWalkMaze
          glasses={upgrades.find(x => x.key == "glasses").owned}
          class={shopMazeClasses}
          height={3}
          width={3}
          onScore={updateScore}
        />
      }</For>
    );
  }
  else if (props.key == "hold-left") {
    return (
      <For each={Array(props.count)}>{() =>
        <HoldLeftMaze
          class={shopMazeClasses}
          height={5}
          width={5}
          onScore={updateScore}
        />
      }</For>
    )
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
  const [mazeSize, setMazeSize] = createStoredSignal("maze-size", 3);

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
          <span class='text-4xl text-center font-extrabold font-pixelated text-white'>{Math.floor(displayedScore())} MAZE FRAGMENTS</span>
        </div>
        <PlayerMaze class='min-h-[70vh] h-full' height={mazeSize()} width={mazeSize()} onScore={(score) => { updateScore(score); setMazeSize(mazeSize() + 1); }} />
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

  let loaded = true;

  onCleanup(() => {
    loaded = false;
    if (audio) {
      audio.onended = undefined;
      audio.pause();
    }
  });

  let canvas: HTMLCanvasElement;

  init().then(() => {
    // let lattice = new SnailLattice(4, 10, 100);

    loaded = true;

    let seed = new Uint16Array(1);
    self.crypto.getRandomValues(seed);

    let start = performance.now();
    let lattice = new SnailLattice(26, 5, 676, seed[0]);
    let end = performance.now();

    console.log(`${end - start}`);

    let [width, height] = lattice.get_dimensions();

    let buffer = new Uint8Array(width * height * 4);

    canvas.width = width;
    canvas.height = height;

    let ctx = canvas.getContext("2d", { alpha: false });

    let prev = performance.now();
    let renderloop = () => {
      if (!loaded) return;

      let now = performance.now();
      let dt = Math.floor((now - prev) * 1000);
      console.log(`FPS: ${1000/(now - prev)}`);
      prev = now;

      lattice.tick(dt*2);
      lattice.render(buffer);

      let imageData = new ImageData(
        new Uint8ClampedArray(buffer),
        width,
        height
      );

      ctx.putImageData(imageData, 0, 0);

      requestAnimationFrame(renderloop);
    }

    requestAnimationFrame(renderloop);
  });

  return (
    <UpgradesProvider>
      <ShopProvider>
        <ScoreProvider>
          <canvas
            class='ml-4 mt-4'
            ref={canvas}
            style={{
              "image-rendering": "pixelated",
            }}
          >
          </canvas>
        </ScoreProvider>
      </ShopProvider>
    </UpgradesProvider>
  );
};
          // <div class='h-screen grid'>
          //   {gameStarted() ? (
          //     <Game />
          //   ) : (
          //     <div class='flex flex-col gap-8 w-96 self-center justify-self-center text-center'>
          //       <h1 class='text-5xl font-extrabold'>Snail Maze</h1>
          //       <button onClick={startGame} class='border-4 font-extrabold text-3xl py-4 px-8 border-black hover:bg-black hover:text-white transition-colors'>Play</button>
          //     </div>
          //   )}
          // </div>

export default App;
