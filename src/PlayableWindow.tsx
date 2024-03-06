import { createSignal } from "solid-js";

const PlayableWindow = () => {
    const [windowShown, setWindowShown] = createSignal(false);

    return (
        <div class="z-ui fixed left-4 bottom-0">
            <div class="overflow-hidden rounded-tl-md rounded-tr-md border-t-4 border-x-4 border-white text-white flex w-[50vh]">
                <button onClick={() => {
                    setWindowShown(!windowShown());
                }} class="p-2 bg-black w-full transition-colors text-start hover:bg-gray-800">
                    <span class="font-extrabold text-lg">SNAIL</span>
                </button>
            </div>

            {windowShown() && <div class="border-t-4 border-x-4 border-white h-[50vh] bg-fg">

            </div>}
        </div>
    );
};

export default PlayableWindow;
