import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  treeshake: true,
  minify: false, // Keep readable for development
  target: 'es2022',
  external: ['zod', '@celebrum-ai/db'],
  tsconfig: 'tsconfig.build.json',
  outDir: 'dist',
  banner: {
    js: `// @celebrum-ai/shared - Generated on ${new Date().toISOString()}`,
  },
});