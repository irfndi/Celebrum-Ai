import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    pool: '@cloudflare/vitest-pool-workers',
    poolOptions: {
      workers: {
        wrangler: {
          configPath: './wrangler.toml',
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@shared': path.resolve(__dirname, '../shared/src'),
      '@arb-edge/shared': path.resolve(__dirname, '../shared/src'),
      '@arb-edge/shared/errors': path.resolve(__dirname, '../shared/src/errors'),
      '@arb-edge/shared/constants': path.resolve(__dirname, '../shared/src/constants'),
      '@arb-edge/shared/types': path.resolve(__dirname, '../shared/src/types'),
    },
  },
  esbuild: {
    target: 'es2022',
  },
});