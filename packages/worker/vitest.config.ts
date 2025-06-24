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
      '@celebrum-ai/shared': path.resolve(__dirname, '../shared/src'),
      '@celebrum-ai/shared/errors': path.resolve(__dirname, '../shared/src/errors'),
      '@celebrum-ai/shared/constants': path.resolve(__dirname, '../shared/src/constants'),
      '@celebrum-ai/shared/types': path.resolve(__dirname, '../shared/src/types'),
    },
  },
  esbuild: {
    target: 'es2022',
  },
});