import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      // Prerendered SEO landings + SPA fallback for converter deep-links.
      fallback: "index.html",
    }),
    prerender: {
      entries: ["*", "/convert"],
    },
  },
};

export default config;
