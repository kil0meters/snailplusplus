import { Component, createSignal } from "solid-js";
import globalPosition from "./state/position";
import { SNAILS, UPGRADES } from "./state/shop";
import { For } from "solid-js";
import { Upgrade } from "./Upgrade";
import { createStore } from "solid-js/store";

const [menuShown, setMenuShown] = createSignal(true);

const HamburgerButton = () => {
    return (
        <button onClick={() => {
            setMenuShown(!menuShown());
        }} role="button" class={`hamburger ${menuShown() ? "clicked" : ""} w-16 h-16 hover:bg-gray-800 transition-colors relative gap-2 bg-black border-white border-t-4 border-b-4 border-l-4`}>
            <span />
            <span />
            <span />
        </button>
    );
};

const SidebarEntry: Component<{ name: keyof typeof SNAILS }> = (props) => {
    const [snail, setSnail] = SNAILS[props.name].store;
    const [snailAnimation, setSnailAnimation] = createSignal(true);

    let isHovering = false;

    const onClick = () => {
        setSnail("count", (count) => count + 1);
    };


    setInterval(() => {
        if (isHovering) {
            setSnailAnimation(x => !x);
        }
    }, 500);

    return (
        <div class="p-4 flex flex-col gap-2" onMouseEnter={() => isHovering = true} onMouseLeave={() => isHovering = false}>
            <div role="button" onClick={onClick} class="pointer rounded-md hover:bg-gray-800 transition-all flex text-white gap-3 select-none items-center">
                <div class="bg-bg w-14 h-14 rounded-md flex items-center justify-center">
                    <div
                        class="w-8 h-8"
                        style={{
                            "background-image": `url("./assets/${props.name}.png")`,
                            "background-repeat": "no-repeat",
                            "background-position": snailAnimation() ? "-1px -3px" : "-33px -3px",
                            "background-size": "64px 32px",
                            "image-rendering": "pixelated"
                        }}
                    />
                </div>

                <div class="flex flex-col">
                    <span class="font-bold text-lg">{SNAILS[props.name].name}</span>
                    <span class="text-gray-200">{SNAILS[props.name].basePrice.toString()} 𝑓</span>
                </div>

                {snail.count && <span class="ml-auto text-2xl font-bold font-mono pr-4">
                    {snail.count}
                </span>}
            </div>
            <div class="flex pl-16">
                <For each={SNAILS[props.name].upgrades.filter(upgrade => !snail.upgrades[upgrade])}>{(upgrade) =>
                    <Upgrade purchased={false} upgrade={upgrade} onClick={() => {
                        setSnail("upgrades", upgrade, true);
                    }} />
                }</For>
            </div>
        </div>
    );
};

const SidebarBody = () => {
    return (
        <div class={`z-ui fixed w-96 right-0 h-full top-[5.5rem] bg-black overflow-hidden border-white border-t-4 border-l-4 flex-grow rounded-tl-xl transition-all duration-300 ${menuShown() ? "" : "translate-x-96 opacity-0"}`}>
            <SidebarEntry name="random-walk" />
            <SidebarEntry name="random-teleport" />
        </div>
    );
};

const ZoomSlider = () => {
    const [globalPos, setGlobalPos] = globalPosition;

    return (<div class={`z-ui fixed bottom-[4.2rem] w-32 right-[21.1rem] transition-all duration-300 ${menuShown() ? "" : "translate-x-96"}`}>
        <input class="rotate-[270deg] cursor-pointer bg-white border-0 appearance-none h-1 accent-white" onInput={e => {
            e.stopPropagation();

            let pos = globalPos();
            pos.scale = Math.pow(+e.target.value, 2);
            setGlobalPos({ ...pos });
        }} type="range" min={Math.sqrt(0.25)} max={Math.sqrt(4)} value={Math.sqrt(globalPos().scale)} step={0.01} />
    </div>
    );
}

const Sidebar = () => {
    return <>
        <ZoomSlider />

        <div class="fixed z-ui right-0 top-3 gap-3 flex w-96 flex-col">
            <div class="flex gap-3 h-16">
                <div class="bg-black border-4 flex-grow border-white rounded-l-xl text-white flex items-center justify-center">
                    <span class="font-extrabold text-2xl select-none">10000 𝑓</span>
                </div>
                <HamburgerButton />
            </div>
        </div>

        <SidebarBody />
    </>;
};

export default Sidebar;
