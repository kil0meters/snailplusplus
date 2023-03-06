import { Accessor, Component, createContext, createSignal, JSX, Setter } from "solid-js";

export const ScoreContext = createContext<[Accessor<bigint>, Setter<bigint>]>();

const ScoreProvider: Component<{ children: JSX.Element }> = (props) => {
    const localScore = localStorage.getItem("score");
    const [score, setScore] = createSignal(localScore ? BigInt(localScore.split(".")[0]) : 0n);

    // save score only periodically, saves a lot of updates
    setInterval(() => {
        localStorage.setItem("score", score().toString());
    }, 500);

    return (
        <ScoreContext.Provider value={[score, setScore]}>
            {props.children}
        </ScoreContext.Provider>
    );
}

export default ScoreProvider;
