import { children, Component, createSignal, For, JSX, Match, onCleanup, onMount, Switch, useContext } from "solid-js";
import { produce } from 'solid-js/store';
import { Portal } from "solid-js/web";
import { NAMES } from "../assets/names";
import Changelog from "./Changelog";
import { LATTICE_WORKER_STORE } from "./Game";
import { ScoreContext } from "./ScoreProvider";
import { SHOP, ShopContext, ShopItem, ShopKey, ShopListing, SHOP_KEYS } from "./ShopProvider";
import { SnailInfoContext } from "./SnailInfoProvider";
import { Upgrade, UPGRADES, UpgradesContext } from "./UpgradesProvider";
import { formatNumber } from "./utils";

const PRICE_SCALER = 1.13;

document["devmode"] = false;

const ShopListingElement: Component<{ key: ShopKey, count: number }> = (props) => {
    const [score, setScore] = useContext(ScoreContext);
    const [_shop, setShop] = useContext(ShopContext);
    const [_snailInfo, setSnailInfo] = useContext(SnailInfoContext);
    const [hover, setHover] = createSignal(false);

    const price = () => BigInt(Math.floor(SHOP[props.key].price * Math.pow(PRICE_SCALER, props.count)));

    const buy = () => {
        if (score() >= price() || document["devmode"]) {
            if (!document["devmode"]) {
                setScore(score() - price());
            }

            setShop(
                (shopItem) => shopItem.key === props.key,
                "count",
                (count) => {
                    setSnailInfo(
                        (info) => info.key == props.key,
                        produce((info) => {
                            while (info.names.length < count + 1) {
                                info.names.push(NAMES[Math.floor(Math.random() * NAMES.length)]);
                                info.createdAts.push(Math.floor(Date.now() / 1000));
                                info.solvedCounts.push(0);
                            }
                        })
                    );

                    return count + 1;
                }
            );

            LATTICE_WORKER_STORE[props.key].postMessage({ type: "alter", diff: 1 });
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
                <span class=''>{formatNumber(price(), true)} fragments</span>
            </div>

            {props.count > 0 && <span class='ml-auto font-extrabold text-3xl self-center'>{props.count}</span>}

            {hover() && <ShopDescription onMouseEnter={() => setHover(false)}>
                <MazeDescription title={SHOP[props.key].name} description={SHOP[props.key].description} fragmentsPerCompletion={SHOP[props.key].baseMultiplier} />
            </ShopDescription>}
        </button>
    );
}

const UpgradeListing: Component<Upgrade & { canBuy: boolean }> = (props) => {
    const [score, setScore] = useContext(ScoreContext);
    const [_upgrades, setUpgrades] = useContext(UpgradesContext);
    const [hover, setHover] = createSignal(false);

    const upgrade = () => UPGRADES[props.key];

    const buy = () => {
        if (!props.owned && props.canBuy && (score() >= upgrade().price || document["devmode"])) {
            if (!document["devmode"])
                setScore(score() - upgrade().price)

            setUpgrades(
                (item) => item.key === props.key,
                "owned",
                () => true
            );
        }
    };

    return (
        <button
            onMouseEnter={() => setHover(true)}
            onMouseLeave={() => setHover(false)}
            onClick={buy}
            class={`p-1 aspect-square ${props.owned || !props.canBuy ? "cursor-default" : ""}`}
        >
            <div
                class={
                    `aspect-square flex items-center justify-center border-4 p-2 transition-all outline-black outline outline-0 ${props.owned ? "bg-black" : "bg-white"} ${props.canBuy ? "border-black hover:outline-4" : "bg-neutral-200"}`
                }
            >
                {upgrade().icon}

                {hover() && <ShopDescription onMouseEnter={() => setHover(false)}>
                    <Switch>
                        <Match when={props.canBuy}>
                            <span class="font-bold text-lg">{upgrade().name}</span>
                            {!props.owned && <span>{formatNumber(upgrade().price, true)} fragments</span>}
                            <span>{upgrade().description}</span>
                        </Match>
                        <Match when={!props.canBuy && upgrade().mazeType != "manual"}>
                            <span class="font-bold text-lg italic">{upgrade().name}</span>
                            <span class="italic">Unlocks after {upgrade().showAfter} {SHOP[upgrade().mazeType].name}s.</span>
                        </Match>
                        <Match when={!props.canBuy && upgrade().mazeType == "manual"}>
                            <span class="font-bold text-lg italic">{upgrade().name}</span>
                            <span class="italic">Unlocks after you have {upgrade().showAfter} units.</span>
                        </Match>
                    </Switch>
                </ShopDescription>}
            </div>
        </button>
    );
}

const MazeDescription: Component<{
    title: string,
    description: string,
    fragmentsPerCompletion: bigint
}> = (props) => {
    return <>
        <h1 class="font-extrabold text-lg">{props.title}</h1>

        <div class="mb-2 flex flex-col gap-1">
            <span class="text-sm">{formatNumber(props.fragmentsPerCompletion, true)} fragments per solve</span>
        </div>

        <span>{props.description}</span>
    </>;
}

const ShopDescription: Component<{
    onMouseEnter: () => void,
    children: JSX.Element
}> = (props) => {
    let hoverContainer: HTMLDivElement;
    let onResize: () => void;
    let onMouseMove: (event: MouseEvent) => void;

    onMount(() => {
        let shopContainer = document.getElementById("shop-sidebar");

        hoverContainer.style.right = `${shopContainer.offsetWidth + 4}px`;

        onResize = () => {
            hoverContainer.style.right = `${shopContainer.offsetWidth + 4}px`;
        };

        onMouseMove = (event: MouseEvent) => {
            let top = Math.max(0, event.clientY - hoverContainer.clientHeight / 2);
            hoverContainer.style.top = `${top}px`;
        };

        addEventListener("resize", onResize);
        addEventListener("mousemove", onMouseMove);
    });

    onCleanup(() => {
        removeEventListener("resize", onResize);
        removeEventListener("resize", onMouseMove);
    })

    return (
        <Portal>
            <div
                ref={hoverContainer}
                onMouseEnter={props.onMouseEnter}
                class="z-50 absolute font-display bg-white p-4 border-4 border-black flex flex-col justify-left text-left w-96"
            >
                {props.children}
            </div>
        </Portal>
    );
};

// this code is so fucking gross. i'm doing so many linear searches through
// for seemingly no reason, but it works i guess and we don't need to do it
// very frequently
const Shop: Component<{ class?: string }> = (props) => {
    const [shop, _setShop] = useContext(ShopContext);
    const [upgrades, _setUpgrades] = useContext(UpgradesContext);

    const purchasedUpgrades = () =>
        upgrades.filter((upgrade) => upgrade.owned);

    const purchasableUpgrades = () =>
        upgrades.filter((upgrade) => {
            if (UPGRADES[upgrade.key].mazeType == "manual") {
                let ownedMazeTypeCount = 0;

                for (let i = 0; i < shop.length; i++) {
                    if (shop[i].count > 0) {
                        ownedMazeTypeCount += 1;
                    } else {
                        break;
                    }
                }

                return !upgrade.owned && ownedMazeTypeCount >= UPGRADES[upgrade.key].showAfter;
            }
            else {
                let shopListing = shop.find((x) => x.key == UPGRADES[upgrade.key].mazeType);
                return !upgrade.owned && shopListing && shopListing.count >= UPGRADES[upgrade.key].showAfter;
            }
        });

    const shownUpgrades = () => {
        let inOtherLists = [...purchasableUpgrades(), ...purchasedUpgrades()];

        return upgrades.filter((upgrade) => {
            if (inOtherLists.find((cur) => cur.key == upgrade.key) != undefined) {
                return false;
            }

            if (UPGRADES[upgrade.key].mazeType == "manual") {
                let order = UPGRADES[upgrade.key].order;

                if (order == 0 && shop[0].count > 0) {
                    return true;
                }

                for (let i = 0; i < upgrades.length; i++) {
                    let upgrade = UPGRADES[upgrades[i].key];
                    if (upgrades[i].owned && upgrade.mazeType == "manual" && upgrade.order == order - 1) {
                        return true;
                    }
                }

                return false;
            }
            else {
                let shopListing = shop.find((x) => x.key == UPGRADES[upgrade.key].mazeType);
                return !upgrade.owned && shopListing && shopListing.count > 0;
            }
        });
    }

    return (
        <div
            id="shop-sidebar"
            class={`${props.class} bg-white overflow-x-hidden overflow-y-auto z-30 fixed top-[30%] bottom-0 left-0 right-0 md:static flex md:flex flex-col shadow-lg border-t-4 md:border-t-0 md:border-l-4 border-black font-display`}>
            <div class='border-b-4 border-black p-4'>
                <h1 class='font-extrabold text-2xl mb-4'>
                    {shop[0].count > 0 ? "Upgrades" : " "}
                </h1>

                <div class='grid grid-cols-7'>
                    <For each={purchasedUpgrades()}>{item =>
                        <UpgradeListing
                            key={item.key}
                            owned={item.owned}
                            canBuy={true}
                        />
                    }</For>
                </div>
                <div class='grid grid-cols-7'>
                    <For each={purchasableUpgrades()}>{item =>
                        <UpgradeListing
                            key={item.key}
                            owned={item.owned}
                            canBuy={true}
                        />
                    }</For>

                    <For each={shownUpgrades()}>{item =>
                        <UpgradeListing
                            key={item.key}
                            owned={item.owned}
                            canBuy={false}
                        />
                    }</For>
                </div>
            </div>

            <For each={shop.filter((_, i, shop) => document["devmode"] || i == 0 || shop[i - 1].count > 0)}>{item => <ShopListingElement
                key={item.key}
                count={item.count}
            />}</For>

            <div class="text-center text-sm text-gray-500 my-8">
                Made with 🐌 · <Changelog /> · <a class="hover:underline text-blue-600" href="https://github.com/kil0meters/snailplusplus" target="_blank">Star on GitHub</a>
            </div>
        </div >
    );
}

export default Shop;
