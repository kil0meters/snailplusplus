import { Component, createEffect, createMemo, createSignal, For, onCleanup, onMount, useContext } from "solid-js";
import { shop, ShopContext, ShopItem, ShopKey, ShopListing } from "./ShopProvider";
import init, { SnailLattice } from "snail-lattice";
import { ScoreContext } from "./ScoreProvider";

const SnailLatticeElement: Component<ShopListing> = (props) => {
  let container: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let loaded = true;
  let lattice: SnailLattice;
  let buffer: Uint8Array;
  let visible = true;

  const [scale, setScale] = createSignal(1);
  const [score, setScore] = useContext(ScoreContext);

  const updateScale = () => {
    const scaleX = container.clientWidth / canvas.width;
    setScale(scaleX);
  }

  onMount(() => {
    const intersectionObserver = new IntersectionObserver(entries => {
      visible = entries[0].isIntersecting;
    }, { threshold: [0] });

    intersectionObserver.observe(container);

    let seed = self.crypto.getRandomValues(new Uint16Array(1))[0];

    let { mazeSize, latticeWidth } = shop[props.key];

    lattice = new SnailLattice(props.key, latticeWidth, mazeSize, props.count, seed);

    let [width, height] = lattice.get_dimensions();

    buffer = new Uint8Array(width * height * 4);

    canvas.width = width;
    canvas.height = height;

    const resizeObserver = new ResizeObserver(updateScale);
    resizeObserver.observe(container);

    let ctx = canvas.getContext("2d", { alpha: true });

    let prev = performance.now();
    let renderloop = () => {
      if (!loaded) return;

      let now = performance.now();
      let dt = Math.floor((now - prev) * 1000);
      prev = now;

      setScore(score() + lattice.tick(dt) * shop[props.key].baseMultiplier);


      if (visible) {
        lattice.render(buffer);

        let imageData = new ImageData(
          new Uint8ClampedArray(buffer),
          canvas.width,
          canvas.height,
        );

        ctx.putImageData(imageData, 0, 0);
      }

      requestAnimationFrame(renderloop);
    }

    requestAnimationFrame(renderloop);
  });

  onCleanup(() => {
    loaded = false;
  });

  createEffect((oldCount: number) => {
    if (lattice) {
      lattice.alter(props.count - oldCount);

      let [width, height] = lattice.get_dimensions();

      buffer = new Uint8Array(width * height * 4);

      canvas.width = width;
      canvas.height = height;

      updateScale();
    }

    return props.count;
  }, props.count);

  return (
    <div ref={container} class={`flex items-center justify-center w-full`}>
      <canvas
        ref={canvas}
        style={{
          "image-rendering": "pixelated",
          width: scale() * canvas.width + "px",
          height: "auto",
        }}
      />
    </div>
  );
}

const SnailLatticeContainer: Component<ShopListing> = (props) => {
  return (
    <>
      {props.count === 0 ? <></> : (
        <>
          <h1 class='p-8 bg-black text-white font-pixelated'>{shop[props.key].name}</h1>
          <div class='px-8 w-full'>
            <SnailLatticeElement key={props.key} count={props.count} />
          </div>
        </>
      )}
    </>
  );
};

const AutoMazes: Component = () => {
  const [shop, _setShop] = useContext(ShopContext);
  const [initialized, setInitialized] = createSignal(false);

  init().then(() => {
    console.log("initailized");
    setInitialized(true);
  });

  return (
    <div class="w-full flex gap-8 flex-col">
      {initialized() && <For each={shop}>
        {item => <SnailLatticeContainer {...item} />}
      </For>}
    </div>
  )
};

export default AutoMazes;
