import { Component, createEffect, createSignal, For, useContext } from "solid-js";
import { Portal } from "solid-js/web";
import { LATTICE_WORKER_STORE } from "./Game";
import { ScoreContext } from "./ScoreProvider";
import { ShopContext, SHOP_KEYS } from "./ShopProvider";
import { SnailInfoContext } from "./SnailInfoProvider";
import { UpgradesContext } from "./UpgradesProvider";

const ChangelogEntry: Component<{ version: string, features: string[], bugfixes: string[] }> = (props) => {
    return (
        <li class="text-lg">
            <b class="text-xl">{props.version}</b><br />
            <i class="text-lg">Features:</i>
            <ul class="list-disc pl-5">
                <For each={props.features}>
                    {(feature) => <li>{feature}</li>}
                </For>
            </ul>
            <i class="text-lg">Bug Fixes:</i>
            <ul class="list-disc pl-5">
                <For each={props.bugfixes}>
                    {(bugfix) => <li>{bugfix}</li>}
                </For>
            </ul>
        </li>
    );
}

const Changelog: Component<{}> = () => {
    const [shown, setShown] = createSignal(false);
    const [shop, setShop] = useContext(ShopContext);
    const [upgrades, setUpgrades] = useContext(UpgradesContext);
    const [score, setScore] = useContext(ScoreContext);
    const [snailInfo, setSnailInfo] = useContext(SnailInfoContext);

    const saveGame = () => {
        return btoa(JSON.stringify({
            shop,
            upgrades,
            score: score().toString(),
            snailInfo,
        }));
    }

    const loadGame = (save: string) => {
        console.log("Loading save: " + atob(save));

        try {
            // TODO: Should probably validate this
            let { shop: newShop, upgrades: newUpgrades, score: newScore, snailInfo: newSnailInfo } = JSON.parse(atob(save));

            setSnailInfo(newSnailInfo);
            setScore(BigInt(newScore));
            setUpgrades(newUpgrades);
            setShop(newShop);
        } catch (e) {
            alert(`error loading game (report this!): ${e}`);
        }
    }

    const reset = () => {
        if (window.confirm("This will cause you to lose all of your progress. Are you sure?")) {
            setShop(() => true, "count", () => 0);
            setUpgrades(() => true, "owned", () => false);
            setScore(0n);

            SHOP_KEYS.forEach((key) => {
                LATTICE_WORKER_STORE[key].postMessage({ type: "reset" });
            });
        }
    };

    let saveGameInput: HTMLInputElement;

    return <>
        {shown() &&
            <Portal>
                <div class="z-50 font-display fixed left-0 top-0 bottom-0 right-0 bg-[#00000055]">
                    <div class="mx-auto max-w-3xl bg-white p-4 border-2 border-black text-black text-left grid grid-cols-3 mt-40">
                        <div class="col-span-2 flex flex-col text-lg gap-4">
                            <h1 class="text-3xl font-extrabold">SnailMaze++</h1>

                            <div>
                                <i>HOW TO PLAY:</i>
                                <p>
                                    Use WASD or arrow keys to control your character. Click on things to purchase them.
                                </p>
                            </div>

                            <div class="grid grid-cols-2 gap-4 mt-auto">
                                <input type="file" ref={saveGameInput} class="hidden" onChange={() => {
                                    const file = saveGameInput.files[0];
                                    const reader = new FileReader();

                                    reader.addEventListener("load", () => {
                                        loadGame(reader.result as string);
                                    });

                                    reader.readAsText(file);
                                }}></input>

                                <button class="rounded-full px-4 py-2 transition-colors bg-neutral-200 hover:bg-neutral-100" onClick={() => {
                                    saveGameInput.click();
                                }}>Load Save</button>
                                <button class="rounded-full px-4 py-2 transition-colors bg-neutral-200 hover:bg-neutral-100" onClick={async () => {
                                    let saveString = saveGame();
                                    const blob = new Blob([saveString], { type: "text/plain;charset=utf-8" });
                                    const link = document.createElement("a");
                                    link.href = URL.createObjectURL(blob);
                                    link.download = "snailpp_save.txt";
                                    document.body.appendChild(link);
                                    link.click();
                                    document.body.removeChild(link);
                                }}>Save game</button>
                            </div>

                            <button onClick={reset} class="rounded-full bg-red-700 py-2 px-4 hover text-red-50 hover:bg-red-600 transition-colors">
                                Reset
                            </button>

                            <button
                                class="bg-neutral-200 hover:bg-neutral-100 px-4 py-2 transition-colors rounded-full"
                                onClick={() => setShown(false)}
                            >Close</button>
                        </div>

                        <div>
                            <h2 class="font-bold text-2xl">Changelog</h2>

                            <ul class="list-disc ml-4">
                                <ChangelogEntry
                                    version="v2.0"
                                    features={[
                                        "Added 4 new manual snails",
                                        "Added 4 new automatic snails",
                                        "Added 11 new upgrades",
                                        "You can now save your game locally.",
                                        "Added changelog :)"
                                    ]}
                                    bugfixes={[
                                        "Average calculations now remain accurate for long-running tabs",
                                        "Fixed shop descriptions not working on Safari",
                                        "Fixed rare crash when upgrading RPG Snail",
                                        "Fixed flicker when regenerating manual maze",
                                    ]}
                                />
                            </ul>
                        </div>
                    </div>
                </div>
            </Portal>
        }
        <button class="hover:underline text-blue-600" onClick={() => setShown((shown) => !shown)}>Changelog</button>
    </>;
}

export default Changelog;
