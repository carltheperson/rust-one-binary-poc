import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import compression from "vite-plugin-compression";

export default defineConfig({
  plugins: [
    sveltekit(),
    compression(), // gzip by default
    compression({ algorithm: "brotliCompress", ext: ".br" }),
  ],
});
