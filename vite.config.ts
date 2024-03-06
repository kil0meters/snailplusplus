import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
// import wasm from "vite-plugin-wasm";
import wasmPack from 'vite-plugin-wasm-pack';
import FullReload from 'vite-plugin-full-reload'

export default defineConfig({
    plugins: [
        solidPlugin(),
        wasmPack(["./snail-lattice"]),
        FullReload(['src/**/*'])
    ],
    server: {
        port: 3000,
    },
    build: {
        target: 'esnext',
    },
});
