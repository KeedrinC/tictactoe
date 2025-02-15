import { Signal, useSignal } from "@preact/signals";
import { GameType } from "../components/types.tsx";

const boardStyle = {
    container: "flex",
    grid: "flex p-4 rounded-xl grid grid-cols-3 gap-4 bg-white",
    square: "flex p-10 rounded-lg bg-gray-300 group text-7xl",
};

export function Board({ game: g }: { game: GameType }) {
    const game = useSignal(g);
    const grid = Array.from({length: 9}).map((_, id) =>
        <Square id={id} game={game} />);
    return (
        <div class={boardStyle.container}>
            <div class={boardStyle.grid}>{grid}</div>
        </div>
    );
}

function Square({ id, game }: { id: number, game: Signal<GameType>}) {
    const { value: { board, playerType } } = game;
    const squareClass = board[id] ? "visible" : "opacity-0 group-hover:opacity-100";
    const squareText = board[id] ? board[id] : playerType;
    const makeMove = () => {
        if (!playerType) return;
        if (!board[id]) board[id] = playerType;
        game.value = { playerType, board };
    }
    return (
        <div class={boardStyle.square} onClick={makeMove}>
            <span class={squareClass}>{squareText}</span>
        </div>
    );
}