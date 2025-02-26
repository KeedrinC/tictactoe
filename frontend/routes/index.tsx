import { Handlers, PageProps } from "$fresh/server.ts";
import Footer from "../components/Footer.tsx";
import GameView from "../islands/Game.tsx";

interface Data { server_url: string }

export const handler: Handlers = {
    GET(_req, ctx) {
        const server_url = Deno.env.get('SERVER_URL');
        return ctx.render({ server_url });
    },
  };

export default function Home({ data }: PageProps<Data>) {
    const { server_url } = data;
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
                    <GameView server_url={server_url} />
                </div>
            </div>
            <Footer class="py-4 mx-auto" />
        </>
    );
}