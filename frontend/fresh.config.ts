import { defineConfig } from "$fresh/server.ts";
import tailwind from "./tailwind/custom_tailwind.ts";

export default defineConfig({
  plugins: [tailwind()],
});
