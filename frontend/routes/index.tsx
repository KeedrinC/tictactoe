import GameView from "../islands/Game.tsx";

export default function Home() {
    return (
        <div class="px-4 py-8 mx-auto bg-[#86efac]">
            <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
                <img
                    class="my-6"
                    src="/logo.svg"
                    width="256"
                    height="256"
                    alt="the Fresh logo: a sliced lemon dripping with juice"
                />
                <GameView />
            </div>
        </div>
    );
}
