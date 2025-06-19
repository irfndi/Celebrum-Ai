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
  external: ['@cloudflare/workers-types'],
  esbuildOptions(options) {
    options.conditions = ['worker', 'browser'];
  },
});