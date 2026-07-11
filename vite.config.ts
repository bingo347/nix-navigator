import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// @ts-expect-error process is a nodejs global
const host: string = process.env.TAURI_DEV_HOST;

export default defineConfig({
    plugins: [svelte({
        configFile: "../svelte.config.ts",
    })],
    clearScreen: false,
    root: "packages",
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? {
            protocol: "ws",
            host,
            port: 1421,
        } : false,
        watch: {
            ignored: ["**/crates/**"],
        },
    },
    resolve: {
        tsconfigPaths: true,
    },
    build: {
        outDir: "../target/@ui",
        target: "esnext",
        rolldownOptions: {
            input: {
                app: "app/app.html",
            },
        }
    },
});