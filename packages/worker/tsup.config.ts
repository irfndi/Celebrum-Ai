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
  noExternal: ['@arb-edge/telegram-bot', '@arb-edge/web', '@arb-edge/shared'],
  external: ['cloudflare:workers'],
  esbuildOptions(options) {
    options.conditions = ['worker', 'browser'];
  },
});