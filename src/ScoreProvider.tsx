import { Accessor, Component, createContext, JSX, Setter } from "solid-js";
import { createStoredSignal } from "./utils";

export const ScoreContext = createContext<[Accessor<number>, Setter<number>]>();

const ScoreProvider: Component<{ children: JSX.Element }> = (props) => {
    const [score, setScore] = createStoredSignal("score", 0);

    return (
        <ScoreContext.Provider value={[score, setScore]}>
            {props.children}
        </ScoreContext.Provider>
    );
}

export default ScoreProvider;
