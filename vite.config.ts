import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
// import wasm from "vite-plugin-wasm";
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  plugins: [
    solidPlugin(),
    wasmPack(["./snail-lattice"])
  ],
  server: {
    port: 3000,
  },
  build: {
    target: 'esnext',
  },
});
