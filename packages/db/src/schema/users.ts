import { sql } from 'drizzle-orm';
import { integer, sqliteTable, text, unique } from 'drizzle-orm/sqlite-core';

export const users = sqliteTable(
  'users',
  {
    id: integer('id', { mode: 'number' }).primaryKey({ autoIncrement: true }),
    telegramId: text('telegram_id').notNull(),
    firstName: text('first_name'),
    lastName: text('last_name'),
    username: text('username'),
    languageCode: text('language_code'),
    email: text('email'),
    role: text('role', { enum: ['free', 'pro', 'ultra', 'admin', 'superadmin'] })
      .notNull()
      .default('free'),
    status: text('status', { enum: ['active', 'suspended', 'banned'] })
      .notNull()
      .default('active'),
    createdAt: integer('created_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
    updatedAt: integer('updated_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
    lastActiveAt: integer('last_active_at', { mode: 'timestamp' }),
    
    // Settings as JSON field - using proper JSON type
    settings: text('settings', { mode: 'json' })
      .$type<{
        notifications?: boolean;
        theme?: 'light' | 'dark';
        language?: string;
        timezone?: string;
      }>()
      .default(sql`'{}'`),
    
    // API limits as JSON field
    apiLimits: text('api_limits', { mode: 'json' })
      .$type<{
        exchangeApis?: number;
        aiApis?: number;
        maxDailyRequests?: number;
      }>()
      .default(sql`'{}'`),
    
    // Account balance
    accountBalance: text('account_balance').default('0.00'),
    betaExpiresAt: integer('beta_expires_at', { mode: 'timestamp' }),
    
    // Trading preferences as JSON field
    tradingPreferences: text('trading_preferences', { mode: 'json' })
      .$type<{
        percentagePerTrade?: number;
        maxConcurrentTrades?: number;
        maxLeverage?: number;
        stopLoss?: number;
        takeProfit?: number;
        riskTolerance?: 'low' | 'medium' | 'high';
        autoTrade?: boolean;
      }>()
      .default(sql`'{}'`),
  },
  (table) => ({
    uniqueTelegramId: unique().on(table.telegramId),
    uniqueEmail: unique().on(table.email),
    uniqueUsername: unique().on(table.username),
  })
);

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;