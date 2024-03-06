import { Component, createSignal } from "solid-js"
import { UPGRADES } from "./state/shop"
import { Popover } from "./Popover"

export const Upgrade: Component<{
    upgrade: keyof typeof UPGRADES,
    purchased: boolean,
    onClick?: (e: MouseEvent) => void
}> = (props) => {
    const [popoverShown, setPopoverShown] = createSignal(false);

    let buttonRef: HTMLButtonElement;

    return (
        <button
            onMouseOver={() => setPopoverShown(true)}
            onMouseOut={() => setPopoverShown(false)}
            onClick={props.onClick} onMouseDown={e => e.stopPropagation()}
            ref={buttonRef}
            class={`px-1 first:pl-0 last:pr-0 ${props.onClick ? "cursor-pointer" : "cursor-default"}`}>

            <div class="rounded bg-gray-800 hover:bg-gray-700 transition-colors w-8 h-8 leading-8">
                {UPGRADES[props.upgrade].icon}
            </div>

            {popoverShown() && <Popover target={buttonRef}>
                <div class="bg-white rounded-md p-4 flex flex-col gap-2 shadow-lg">
                    <div class="flex items-center">
                        <span class="font-bold text-lg">{UPGRADES[props.upgrade].name}</span>
                        {!props.purchased && <span class="ml-auto">{UPGRADES[props.upgrade].price.toString()}𝑓</span>}
                    </div>
                    <span>{UPGRADES[props.upgrade].description}</span>
                </div>
            </Popover>}
        </button>
    );
}
