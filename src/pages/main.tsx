import { onMount } from "solid-js";
import globalPosition from "../state/position";
import { start } from "../engine";
import Sidebar from "../Sidebar";
import PlayableWindow from "../PlayableWindow";
import { DraggableWindow, SnailWindow } from "../SnailWindow";

const Main = () => {
    const [globalPos, setGlobalPos] = globalPosition;

    let isDragging = false;

    const onMouseDown = (e: MouseEvent) => {
        e.preventDefault();

        if (e.button === 1) {
            document.body.style.cursor = 'grabbing';
            isDragging = true;
        }
    };

    const onMouseUp = (e: MouseEvent) => {
        if (e.button === 1) {
            e.preventDefault();

            document.body.style.cursor = 'default';
            isDragging = false;
        }
    };

    const onMouseMove = (e: MouseEvent) => {
        if (isDragging) {
            let curPos = globalPos();
            setGlobalPos({
                x: e.movementX / (curPos.scale * devicePixelRatio * 0.5) + curPos.x,
                y: e.movementY / (curPos.scale * devicePixelRatio * 0.5) + curPos.y,
                scale: curPos.scale
            });
        }
    };

    window.onmousemove = onMouseMove;
    window.onmouseup = onMouseUp;

    onMount(() => {
        start();
    });

    addEventListener("wheel", (e) => {
        let curPos = globalPos();

        curPos.scale = Math.min(Math.max(curPos.scale - 0.1 * curPos.scale * Math.sign(e.deltaY), 0.25), 4);
        setGlobalPos({ ...curPos });
    });

    return <>
        <canvas id="canvas" class="fixed w-screen h-screen z-canvas pointer-events-none" />

        <Sidebar />

        <PlayableWindow />

        <div
            class="bg-gray-500 w-screen h-screen overflow-hidden"
            onMouseDown={onMouseDown}>
            <div
                style={{
                    transform: `scale(${globalPos().scale})`
                }}>

                <SnailWindow name="random-walk" />
                <SnailWindow name="random-teleport" />
            </div>
        </div>
    </>;
};

export default Main;
