import { Component, createSignal, For, useContext } from "solid-js";
import { LATTICE_WORKER_STORE } from "./Game";
import { ScoreContext } from "./ScoreProvider";
import { ShopContext, SHOP_KEYS } from "./ShopProvider";
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
    const [_shop, setShop] = useContext(ShopContext);
    const [_upgrades, setUpgrades] = useContext(UpgradesContext);
    const [_score, setScore] = useContext(ScoreContext);

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

    return <>
        {shown() && <div class="fixed left-0 top-0 bottom-0 right-0 bg-[#00000055]">
            <div class="mx-auto max-w-3xl bg-white p-4 border-2 border-black text-black text-left grid grid-cols-3 mt-40">
                <div class="col-span-2 flex flex-col text-lg">
                    <h1 class="text-3xl font-extrabold mb-2">SnailMaze++</h1>

                    <i>HOW TO PLAY:</i>
                    <p>
                        Use WASD or arrow keys to control your character. Click on things to purchase them.
                    </p>

                    <button onClick={reset} class="mt-auto rounded-full bg-red-700 py-2 px-4 mb-4 hover text-red-50 hover:bg-red-600 transition-colors">
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
                            version="v2.0-dev"
                            features={[
                                "Added 3 new manual snails",
                                "Added 10 new automatic snails",
                                "Added 30 new upgrades",
                                "Added changelog :)"
                            ]}
                            bugfixes={[
                                "Average calculations now remain accurate for long-running tabs",
                                "Fixed flicker when regenerating maze",
                            ]}
                        />
                    </ul>
                </div>
            </div>
        </div>}
        <button class="hover:underline text-blue-600" onClick={() => setShown((shown) => !shown)}>Changelog</button>
    </>;
}

export default Changelog;
