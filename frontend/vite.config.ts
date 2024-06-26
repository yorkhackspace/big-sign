import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: "../static",
    emptyOutDir: true
  },
  server: {
    host: '127.0.0.1'
  }
})
