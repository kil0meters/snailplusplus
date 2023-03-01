import { Component, createSignal, For, onCleanup, onMount, useContext } from "solid-js";
import { LATTICE_WORKER_STORE } from "./Game";
import { ScoreContext } from "./ScoreProvider";
import { SHOP, ShopContext, ShopItem, ShopListing } from "./ShopProvider";
import { Upgrade, upgrades, UpgradesContext } from "./UpgradesProvider";

const PRICE_SCALER = 1.15;

const ShopListingElement: Component<ShopListing> = (props) => {
    const [score, setScore] = useContext(ScoreContext);
    const [_shop, setShop] = useContext(ShopContext);
    const [hover, setHover] = createSignal(false);

    const price = () => Math.floor(SHOP[props.key].price * Math.pow(PRICE_SCALER, props.count));

    const buy = () => {
        if (score() >= price() || true) {
            // LATTICE_STORE[props.key].push();

            // for (let i = 0; i < 1000; i++) {
            setShop(
                (shopItem) => shopItem.key === props.key,
                "count",
                (count) => count + 1
            );

            LATTICE_WORKER_STORE[props.key].postMessage({ type: "alter", diff: 1 });
            // }

            // setScore(score() - price());
        }
    };

    return (
        <button
            onMouseEnter={() => setHover(true)}
            onMouseLeave={() => setHover(false)}
            onClick={buy}
            class='flex hover:bg-neutral-100 p-4 transition-colors text-left'>
            <div class='flex flex-col'>
                <span class='text-2xl font-extrabold'>{SHOP[props.key].name}</span>
                <span class=''>{price()} fragments</span>
            </div>

            {props.count > 0 && <span class='ml-auto font-extrabold text-3xl self-center'>{props.count}</span>}

            {hover() && <ShopDescription onMouseEnter={() => setHover(false)} title={SHOP[props.key].name} description={SHOP[props.key].description} multiplier={SHOP[props.key].baseMultiplier} />}
        </button>
    );
}

// const UpgradeListing: Component<Upgrade> = (props) => {
//     const [score, setScore] = useContext(ScoreContext);
//     const [_shop, setUpgrades] = useContext(UpgradesContext);
//     const [hover, setHover] = createSignal(false);
//
//
//     return;
//     let upgrade = upgrades[props.key];
//
//     const buy = () => {
//         if (props.owned) {
//             setScore(score() + upgrade.price)
//             setUpgrades(
//                 (item) => item.key === props.key,
//                 "owned",
//                 () => false
//             );
//         } else {
//             if (score() >= upgrade.price) {
//                 setScore(score() - upgrade.price)
//                 setUpgrades(
//                     (item) => item.key === props.key,
//                     "owned",
//                     () => true
//                 );
//             }
//         }
//     };
//
//     return <>
//         <button
//             onMouseEnter={() => setHover(true)}
//             onMouseLeave={() => setHover(false)}
//             onClick={buy}
//             class={
//                 `border-4 border-black p-2 transition-all outline-black outline outline-0 hover:outline-4 ${props.owned ? "bg-black text-white" : "bg-white text-black"}`
//             }>
//             {upgrade.name}
//
//             {hover() && <ShopDescription title={upgrade.name} description={upgrade.description} />}
//         </button>
//     </>
// }

const ShopDescription: Component<{
    onMouseEnter: () => void,
    title: string,
    description: string,
    multiplier?: number
}> = (props) => {
    let hoverContainer: HTMLDivElement;
    let onResize: () => void;
    let onMouseMove: (event: MouseEvent) => void;

    onMount(() => {
        let shopContainer = document.getElementById("shop-sidebar");

        hoverContainer.style.right = `${shopContainer.clientWidth}px`;

        onResize = () => {
            hoverContainer.style.right = `${shopContainer.clientWidth}px`;
        };

        onMouseMove = (event: MouseEvent) => {
            hoverContainer.style.top = `${event.clientY - hoverContainer.clientHeight / 2}px`;
        };

        addEventListener("resize", onResize);
        addEventListener("mousemove", onMouseMove);
    });

    onCleanup(() => {
        removeEventListener("resize", onResize);
        removeEventListener("resize", onMouseMove);
    })

    return (
        <div
            onMouseEnter={props.onMouseEnter}
            ref={hoverContainer}
            class="absolute bg-white p-4 border-4 border-black flex flex-col gap justify-left text-left w-96"
        >
            <h1 class="font-extrabold text-lg">{props.title}</h1>
            <span>{props.description}</span>

            {props.multiplier && <div>
                <span class="font-bold">Multiplier: </span>
                <span>{props.multiplier}x</span>
            </div>}
        </div>
    );
};

const Shop: Component = () => {
    const [shop, setShop] = useContext(ShopContext);
    const [_score, setScore] = useContext(ScoreContext);
    const [upgrades, _setUpgrades] = useContext(UpgradesContext);

    const reset = () => {
        setShop(() => true, "count", () => 0);
        setScore(0);

        shop.forEach(({ key }) => {
            LATTICE_WORKER_STORE[key].postMessage({ type: "reset" });
        });
    };

    return (
        <div
            id="shop-sidebar"
            class="bg-white overflow-hidden flex flex-col shadow-lg border-l-4 border-black font-display">
            {/*<div class='border-b-4 border-black p-4'>
                <h1 class='font-extrabold text-2xl mb-4'>Upgrades</h1>

                <div class='flex gap-4'>
                    <For each={upgrades}>{item =>
                        <UpgradeListing
                            key={item.key}
                            owned={item.owned}
                        />
                    }</For>
                </div>
            </div>*/}

            <For each={shop}>{item => <ShopListingElement
                key={item.key}
                count={item.count}
            />}</For>

            <button onClick={reset} class="bg-red-700 p-4 hover text-red-50 hover:bg-red-600 transition-colors">
                Reset
            </button>
        </div>
    );
}

export default Shop;
