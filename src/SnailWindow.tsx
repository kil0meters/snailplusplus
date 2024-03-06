import { Component, For, JSX, createSignal } from "solid-js";
import newZIndex from "./zIndexManager";
import globalPosition from "./state/position";
import { SnailKey, SNAILS } from "./state/shop";
import { Upgrade } from "./Upgrade";

const Positioned: Component<{
    children?: JSX.Element,
    class?: string,
    x: number,
    y: number,
}> = (props) => {
    const [globalPos, _] = globalPosition;
    let [zIndex, setZIndex] = createSignal(newZIndex());

    const perspectiveX = () => globalPos().x + props.x;
    const perspectiveY = () => globalPos().y + props.y;

    return (
        <div
            onMouseDown={() => {
                setZIndex(newZIndex());
            }}
            class={`positioned min-w-max ${props.class}`}
            style={{
                position: 'absolute',
                "z-index": zIndex(),
                transform: `translate(${perspectiveX()}px, ${perspectiveY()}px)`
            }}>
            {props.children}
        </div>
    );
};

type Dimensions = { width: number, height: number };
type Position = { x: number, y: number };

export const DraggableWindow: Component<{
    title: JSX.Element,
    stepSize: number, // step size in pixels of resize bound
    minSize: number, // minimum size in steps
    sizeOffset: number,
    defaultPosition: Position,
    defaultDimensions: Dimensions,
    onMove: (pos: Position) => void,
    onResize: (dimensions: Dimensions) => void,
    children?: JSX.Element
}> = (props) => {
    const [pos, setPos] = createSignal(props.defaultPosition);
    const [globalPos, _] = globalPosition;

    // dimensions represent the size, in pixels, of the internal display window
    const [dimensions, setDimensions] = createSignal(props.defaultDimensions);

    const windowDrag = (e: MouseEvent) => {
        const onMouseMove = (e: MouseEvent) => {
            let curPos = pos();
            setPos({
                x: e.movementX / (globalPos().scale * devicePixelRatio * 0.5) + curPos.x,
                y: e.movementY / (globalPos().scale * devicePixelRatio * 0.5) + curPos.y
            });
            props.onMove(pos());
        };

        if (e.button === 0) {
            window.addEventListener('mousemove', onMouseMove);
            window.addEventListener('mouseup', (_) => {
                window.removeEventListener('mousemove', onMouseMove);
            }, { once: true });
        }

    };

    const resizeDrag = (
        verticalLocked: boolean,
        horizontalLocked: boolean,
        topAdjust: boolean,
        leftAdjust: boolean,
    ) => {
        return (e: MouseEvent) => {
            let totalXMovement = 0;
            let totalYMovement = 0;

            const onMouseMove = (e: MouseEvent) => {
                if (!horizontalLocked)
                    totalXMovement += e.movementX / (globalPos().scale * devicePixelRatio * 0.5);
                if (!verticalLocked)
                    totalYMovement += e.movementY / (globalPos().scale * devicePixelRatio * 0.5);

                while (Math.abs(totalXMovement) >= props.stepSize) {
                    let moveAmount = Math.sign(totalXMovement);
                    totalXMovement -= moveAmount * props.stepSize;

                    if (leftAdjust) {
                        let curPos = pos();
                        setPos({ x: curPos.x + moveAmount * props.stepSize, y: curPos.y })
                        moveAmount *= -1;
                    }

                    let curDimensions = dimensions();
                    setDimensions({ width: Math.max(curDimensions.width + moveAmount, props.minSize), height: curDimensions.height })
                    props.onResize(dimensions());
                }

                while (Math.abs(totalYMovement) >= props.stepSize) {
                    let moveAmount = Math.sign(totalYMovement);
                    totalYMovement -= moveAmount * props.stepSize;

                    if (topAdjust) {
                        let curPos = pos();
                        setPos({ x: curPos.x, y: curPos.y + moveAmount * props.stepSize })
                        moveAmount *= -1;
                    }

                    let curDimensions = dimensions();
                    setDimensions({ width: curDimensions.width, height: Math.max(curDimensions.height + moveAmount, 1) })
                    props.onResize(dimensions());
                }
            };

            if (e.button === 0) {
                window.addEventListener('mousemove', onMouseMove);
                window.addEventListener('mouseup', (_) => {
                    window.removeEventListener('mousemove', onMouseMove);
                }, { once: true });
            }
        };
    }

    return (
        <Positioned
            data-width="10"
            data-height="10"
            class="select-none inline-grid rounded-md overflow-hidden grid-rows-[12px_1fr_12px] grid-cols-[12px_1fr_12px]" x={pos().x} y={pos().y}>
            <span onMouseDown={resizeDrag(false, false, true, true)} class="cursor-nwse-resize pt-[8px] pl-[8px]"><div class="bg-fg w-full h-full" /></span>
            <span onMouseDown={resizeDrag(false, true, true, true)} class="cursor-row-resize pt-[8px]"><div class="bg-fg w-full h-full" /></span>
            <span onMouseDown={resizeDrag(false, false, true, false)} class="cursor-nesw-resize pt-[8px] pr-[8px]"><div class="bg-fg w-full h-full" /></span>

            <span onMouseDown={resizeDrag(true, false, false, true)} class="cursor-col-resize pl-[8px]" left-edge><div class="bg-fg w-full h-full" /></span>
            <div class="flex flex-col overflow-hidden">
                <div class="bg-black p-2 leading-8 font-extrabold text-white border-b-4 border-fg cursor-default active:cursor-grabbing" onMouseDown={windowDrag}>
                    {props.title}
                </div>

                <div class="bg-bg grid" style={{
                    width: `${dimensions().width * props.stepSize - props.sizeOffset}px`,
                    height: `${dimensions().height * props.stepSize - props.sizeOffset}px`,
                }}>
                    {props.children}
                </div>
            </div>
            <span onMouseDown={resizeDrag(true, false, false, false)} class="cursor-col-resize pr-[8px]" right-edge><div class="bg-fg w-full h-full" /></span>

            <span onMouseDown={resizeDrag(false, false, false, true)} class="cursor-nesw-resize pb-[8px] pl-[8px]" bottom-left-corner><div class="bg-fg w-full h-full" /></span>
            <span onMouseDown={resizeDrag(false, true, false, false)} class="cursor-row-resize pb-[8px]" top-edge><div class="bg-fg w-full h-full" /></span>
            <span onMouseDown={resizeDrag(false, false, false, false)} class="cursor-nwse-resize pb-[8px] pr-[8px]" bottom-left-corner><div class="bg-fg w-full h-full" /></span>
        </Positioned>
    );
}

export const SnailWindow: Component<{ name: SnailKey }> = (props) => {
    const [snail, setSnail] = SNAILS[props.name].store;

    const stepSize = () => (SNAILS[props.name].size + 0.1) * 20;

    return (
        <DraggableWindow
            title={
                <div class="flex items-center">
                    <span class="h-8">{SNAILS[props.name].name}</span>

                    <div class="ml-auto flex">
                        <For each={SNAILS[props.name].upgrades.filter(upgrade => snail.upgrades[upgrade])}>{(upgrade) =>
                            <Upgrade purchased={true} upgrade={upgrade} />
                        }</For>
                    </div>
                </div>
            }
            stepSize={stepSize()}
            minSize={SNAILS[props.name].minWidth}
            sizeOffset={2}
            onMove={(pos) => {
                setSnail(pos);
            }}
            onResize={(dimensions) => {
                setSnail(dimensions);
            }}
            defaultPosition={{ x: snail.x, y: snail.y }}
            defaultDimensions={{ width: snail.width, height: snail.height }}
        >
            <div id={props.name} data-mazesize={SNAILS[props.name].size + 0.1} class="viewport" />
        </DraggableWindow>
    );
}
