import { useEffect, useState } from "preact/hooks";
import { Lobby, Session } from "../components/types.tsx";
import Board from "./Board.tsx";
import LobbyView from "./Lobby.tsx";
import Game from "../lib/Game.tsx";

function setup(on_message: (socket: WebSocket, data: { data: string }) => void): WebSocket {
    const url = new URL("/ws", location.origin.replace("http", "ws"));
    url.port = "80";
    const socket = new WebSocket(url);
    socket.onopen = () => {
        console.log("Connected to WebSocket server");
        socket.send(JSON.stringify({
            "type": "Connection",
            "data": { "nickname": "keedrin" }
        }));
    };
    socket.onmessage = (data) => on_message(socket, data);
    socket.onerror = (error) => console.error("WebSocket Error:", error);
    socket.onclose = () => console.log("WebSocket closed");
    return socket;
}

export default function GameView() {
    const [error, setError] = useState<string>();
    const [_, setSession] = useState<Session>();
    const [lobby, setLobby] = useState<Lobby>();
    const [game, setGame] = useState<Game>();
    const [socket, setSocket] = useState<WebSocket>();
    useEffect(() => { // establish websocket connection and handle messages
        const ws = setup((socket, { data: d }) => {
            const response = JSON.parse(d);
            console.log("Message from server:", response)
            const { type, data } = response;
            if (type == "Error") {
                console.log("error message: ", data);
                setError(data);
            }
            if (type == "Session") setSession(data as Session);
            if (type == "Lobby") setLobby(data as Lobby);
            if (type == "StartGame") setGame(new Game(socket, data.player));
            if (type == "Move") setGame(() => {
                if (!game) return;
                const { position, player } = data;
                game.board[position] = player;
                return game;
            });
        });
        setSocket(ws);
    }, []);
    return (
        <div class="flex flex-col gap-4 justify-items-center w-sm max-w-sm min-w-sm">
            {socket && <LobbyView lobby={lobby} socket={socket} error={error} /> }
            <Board socket={socket} game={game} />
        </div>
    );
}