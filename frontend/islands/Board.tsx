import { Signal, useSignal } from "@preact/signals";
import Game from "../lib/Game.tsx";

const boardStyle = {
    grid: "flex p-3 rounded-xl grid grid-cols-3 gap-3 bg-white",
    square: "flex p-7 rounded-lg bg-gray-300 group text-7xl",
};

export default function Board({ socket, game: g }: { socket?: WebSocket, game?: Game }) {
    const game = useSignal(g);
    const grid = Array.from({length: 9}).map((_, position) =>
        <Square key={position} position={position} game={game} socket={socket} />);
    return <div class={boardStyle.grid}>{grid}</div>;
}

interface SquareProps {
    socket?: WebSocket,
    position: number,
    game?: Signal<Game | undefined>
}

function Square({ socket, position, game }: SquareProps) {
    const { board, player } = game?.value || {};
    const squareClass = board?.[position] ? "visible" : game?.value ? "opacity-0 group-hover:opacity-100" : "invisible";
    const squareText = board?.[position] ?? player ?? "-";
    const handleMove = () => {
        if (!(game?.value && socket)) return;
        const request = { type: "Move", position };
        socket.send(JSON.stringify(request));
    }
    return (
        <div class={boardStyle.square} onClick={handleMove}>
            <span class={squareClass}>{squareText}</span>
        </div>
    );
}