import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm'],
  dts: true,
  sourcemap: true,
  clean: true,
  outDir: 'dist',
  external: ['@cloudflare/workers-types', 'drizzle-orm', 'drizzle-kit'],
  splitting: false,
  treeshake: false,
});