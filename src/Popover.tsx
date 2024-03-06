import { Component, JSX } from "solid-js"
import { Portal } from "solid-js/web"

export const Popover: Component<{
    children: JSX.Element,
    target: Element,
}> = (props) => {
    const rect = () => props.target.getBoundingClientRect();

    const POPOVER_WIDTH = 240;

    return (
        <Portal>
            <div class="fixed z-ui pt-2" style={{
                width: `${POPOVER_WIDTH}px`,
                top: `${rect().top + rect().height}px`,
                left: `${rect().left + (rect().width - POPOVER_WIDTH) / 2}px`
            }}>
                {props.children}
            </div>
        </Portal>
    );
}
