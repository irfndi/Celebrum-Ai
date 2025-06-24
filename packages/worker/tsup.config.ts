import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  target: 'es2022',
  outDir: 'dist',
  clean: true,
  minify: true,
  sourcemap: true,
  dts: false,
  external: ['@celebrum-ai/shared', '@celebrum-ai/db', 'cloudflare:workers'],
  esbuildOptions(options) {
    options.conditions = ['worker', 'browser'];
  },
});