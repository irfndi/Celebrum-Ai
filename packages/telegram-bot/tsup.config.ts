import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    'types/index': 'src/types/index.ts',
    'handlers/index': 'src/handlers/index.ts'
  },
  format: ['esm'],
  dts: true,
  clean: true,
  splitting: false,
  treeshake: false
});