import { defineConfig } from 'tsup';

export default defineConfig({
  entry: [
    'src/index.ts',
    'src/types/index.ts',
    'src/utils/index.ts',
    'src/config/index.ts',
    'src/constants/index.ts',
    'src/errors/index.ts',
    'src/validation/index.ts',
    'src/middleware/index.ts',
  ],
  format: ['esm', 'cjs'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  treeshake: true,
  minify: false, // Keep readable for development
  target: 'es2022',
  external: ['zod'],
  outDir: 'dist',
  banner: {
    js: `// @arb-edge/shared - Generated on ${new Date().toISOString()}`,
  },
});