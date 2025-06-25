import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  target: 'node20',
  platform: 'node',
  outDir: 'dist',
  clean: true,
  minify: true,
  sourcemap: true,
  dts: false,
  external: ['@celebrum-ai/shared', '@celebrum-ai/db'],
});