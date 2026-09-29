import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    manifest: true,
    lib: {
      entry: resolve(import.meta.dirname, 'assets/app.ts'),
      formats: ["es"],
      cssFileName: "app"
    },
    rollupOptions: {
      output: {
        // Appends the digest hash to your JS entry file
        entryFileNames: '[name].[hash].js',

        // Appends the digest hash to split vendor/lazy-loaded chunks
        chunkFileNames: '[name].[hash].js',

        // Appends the digest hash to CSS and static media files
        assetFileNames: '[name].[hash][extname]'
      }
    }
  }
})