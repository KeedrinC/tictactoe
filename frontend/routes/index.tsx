import Footer from "../components/Footer.tsx";
import GameView from "../islands/Game.tsx";

export default function Home() {
    return (
        <>
            <div class="px-4 py-8 mx-auto bg-[#86efac]">
                <div class="max-w-screen-md mx-auto flex flex-col items-center justify-center">
                    <img
                        class="mb-6"
                        src="/logo.svg"
                        width="256"
                        height="256"
                        alt="the Fresh logo: a sliced lemon dripping with juice"
                    />
                    <GameView />
                </div>
            </div>
            <Footer class="py-4 mx-auto" />
        </>
    );
}