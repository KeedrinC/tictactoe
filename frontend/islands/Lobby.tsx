import { Dispatch, StateUpdater, useEffect, useState } from "preact/hooks";
import { Lobby } from "../lib/types.tsx";

enum CurrentView { JoinLobby, LobbyDetails }

export default function LobbyView({ socket, lobby, error, server_url }: { socket: WebSocket, lobby?: Lobby, error?: string, server_url: string }) {
    const [view, setView] = useState<CurrentView>(
        // CurrentView.JoinLobby
        // CurrentView.LobbyDetails
    );

    useEffect(() => {
        if (lobby) setView(CurrentView.LobbyDetails);
    }, [lobby]);

    switch (view) {
        case CurrentView.JoinLobby:
            return <JoinLobby socket={socket} setView={setView} error={error} />;
        case CurrentView.LobbyDetails:
            return <LobbyDetails lobby={lobby!} error={error} server_url={server_url} />;
        default:
            return <ActionButtons socket={socket} setView={setView} error={error} />;
    }
}

const buttonStyle = "grow p-4 rounded-lg bg-slate-50 dark:bg-slate-800 font-bold text-white text-center";
const Button = ({ class: c, text, onClick }: { class?: string, text: string, onClick?: () => void }) =>
    <button type="button" class={`${buttonStyle} ${c}`} onClick={onClick}>{text}</button>;

function JoinLobby({ socket, setView }: { socket: WebSocket, setView: Dispatch<StateUpdater<CurrentView | undefined>>, error?: string }) {
    const [code, setCode] = useState<string | undefined>();
    return (
        <div class="flex flex-col gap-1">
            <div class="flex">
                <p class="text-2xl font-bold my-auto">Join Lobby</p>
                <button type="button" class="p-2 ml-auto rounded-lg text-base font-medium" onClick={() => setView(undefined)}>Cancel</button>
            </div>
            <div class="flex gap-3">
                <input
                    type="text"
                    name="code"
                    id="code"
                    class="rounded-lg min-w-0 grow py-4 pr-3 pl-3 bg-slate-50 text-gray-900 placeholder:text-gray-400 focus:outline-none sm:text-sm/6 font-bold placeholder:font-normal placeholder:text-lg"
                    placeholder="Enter 4 digit lobby code"
                    onInput={(event) => setCode(event.currentTarget.value)}
                ></input>
                <Button text="ENTER" onClick={() => {
                    const request = { type: "JoinLobby", data: { code }};
                    if (code) socket.send(JSON.stringify(request));
                }} />
            </div>
        </div>
    )
}

function LobbyDetails({ lobby, server_url }: { lobby: Lobby, error?: string, server_url: string }) {
    const { code } = lobby ?? { code: "1234"};
    const CodeView = Array.from(code).map((n, i) =>
        <span key={i} class="flex rounded-lg p-3 bg-slate-800 text-white font-bold my-auto">{n}</span>);
    return (
        <div class="flex gap-3">
            <div class="flex gap-1">{CodeView}</div>
            <div>
                <p class="flex text-sm/4">
                    Have a friend enter this 4 digit code
                    or send them this quick link.
                </p>
                <a href={`${server_url}/#${code}`}>
                    {server_url}/#{code}
                </a>
            </div>
        </div>
    )
}

interface ActionButtonProps {
    socket: WebSocket,
    setView: Dispatch<StateUpdater<CurrentView | undefined>>,
    error?: string
}
function ActionButtons({ socket, setView }: ActionButtonProps) {
    return (
        <div class="flex gap-3">
            <Button text="CREATE LOBBY" onClick={() => {
                const request = { type: "CreateLobby" };
                socket.send(JSON.stringify(request));
            }}/>
            <Button text="JOIN LOBBY" onClick={() => setView(CurrentView.JoinLobby)}/>
        </div>
    )
}