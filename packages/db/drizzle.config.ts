import type { Config } from 'drizzle-kit';

export default {
  dialect: 'sqlite',
  schema: './src/schema/index.ts',
  out: './sql/migrations',
  driver: 'd1-http',
  dbCredentials: {
    // These will be populated from environment/wrangler.toml
    accountId: process.env.CLOUDFLARE_ACCOUNT_ID!,
    databaseId: process.env.CLOUDFLARE_D1_DATABASE_ID!,
    token: process.env.CLOUDFLARE_D1_TOKEN!,
  },
  verbose: true,
  strict: true,
} satisfies Config; 