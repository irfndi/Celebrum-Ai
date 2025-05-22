import { defineWorkersConfig } from "@cloudflare/vitest-pool-workers/config";
import path from "node:path";

export default defineWorkersConfig({
  // Top-level resolve configuration
  resolve: {
    alias: {
      "supports-color": path.resolve(__dirname, "tests/__mocks__/supports-color.js"),
      "@colors/colors/safe": path.resolve(__dirname, "tests/__mocks__/colorsSafe.js"),
      // Explicitly alias logform to its mock file
      "logform": path.resolve(__dirname, "tests/__mocks__/logform.js"),
      "winston": path.resolve(__dirname, "tests/__mocks__/winston.js"),
      "ccxt": path.resolve(__dirname, "tests/__mocks__/ccxt.ts"),
      // "os": path.resolve(__dirname, "tests/__mocks__/node-os.cjs"), // Removed
      // "node:os": path.resolve(__dirname, "tests/__mocks__/node-os.js"), // Removed
    },
  },
  test: {
    globals: true,
    setupFiles: ['./tests/vitest.setup.ts'],
    poolOptions: {
      workers: {
        wrangler: { configPath: "./wrangler.toml" },
        // You can add other Miniflare options here if needed
        // miniflare: {
        //   liveReload: true,
        // },
      },
    },
    server: {
      deps: {
        inline: [],
      },
    },
    coverage: {
      provider: "istanbul",
      reporter: ["text", "lcov"],
      thresholds: {
        statements: 95,
        branches: 95,
        functions: 95,
        lines: 95,
      },
    },
  },
});
