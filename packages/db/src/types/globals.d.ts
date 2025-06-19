/// <reference types="@cloudflare/workers-types" />

declare global {
  // Re-export D1Database type globally for easier usage
  type D1Database = import("@cloudflare/workers-types").D1Database;
}

export {}; 