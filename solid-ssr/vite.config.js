import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// https://vite.dev/config/
export default defineConfig({
  ssr: {
    noExternal: ["solid-js", "solid-js/web"],
  },
  plugins: [solid({ ssr: true })],
});
