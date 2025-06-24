import { sql } from 'drizzle-orm';
import { integer, real, sqliteTable, text, index } from 'drizzle-orm/sqlite-core';
import { users } from './users';

export const positions = sqliteTable(
  'positions',
  {
    id: integer('id', { mode: 'number' }).primaryKey({ autoIncrement: true }),
    userId: integer('user_id')
      .notNull()
      .references(() => users.id, { onDelete: 'cascade' }),
    exchangeId: text('exchange_id').notNull(),
    symbol: text('symbol').notNull(),
    type: text('type', { enum: ['long', 'short'] }).notNull(),
    strategy: text('strategy', { enum: ['arbitrage', 'technical', 'manual'] }).notNull(),
    entryPrice: real('entry_price').notNull(),
    exitPrice: real('exit_price'),
    quantity: real('quantity').notNull(),
    leverage: real('leverage').default(1),
    stopLoss: real('stop_loss'),
    takeProfit: real('take_profit'),
    status: text('status', { enum: ['open', 'closed', 'partially_filled', 'cancelled'] })
      .notNull()
      .default('open'),
    pnl: real('pnl').default(0),
    fees: real('fees').default(0),
    
    // Metadata as JSON field
    metadata: text('metadata', { mode: 'json' })
      .$type<{
        fundingRate?: number;
        correlatedPositions?: string[];
        riskScore?: number;
        autoClose?: boolean;
      }>()
      .default(sql`'{}'`),
    
    createdAt: integer('created_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
    updatedAt: integer('updated_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
    closedAt: integer('closed_at', { mode: 'timestamp' }),
  },
  (table) => ({
    userIdIdx: index('positions_user_id_idx').on(table.userId),
    statusIdx: index('positions_status_idx').on(table.status),
    symbolIdx: index('positions_symbol_idx').on(table.symbol),
    strategyIdx: index('positions_strategy_idx').on(table.strategy),
  })
);

export const opportunities = sqliteTable(
  'opportunities',
  {
    id: integer('id', { mode: 'number' }).primaryKey({ autoIncrement: true }),
    type: text('type', { enum: ['arbitrage', 'technical'] }).notNull(),
    symbol: text('symbol').notNull(),
    exchange1: text('exchange_1').notNull(),
    exchange2: text('exchange_2').notNull(),
    price1: real('price_1').notNull(),
    price2: real('price_2').notNull(),
    profitPercentage: real('profit_percentage').notNull(),
    confidence: real('confidence').notNull(),
    expiresAt: integer('expires_at', { mode: 'timestamp' }).notNull(),
    isActive: integer('is_active', { mode: 'boolean' }).notNull().default(true),
    createdAt: integer('created_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
  },
  (table) => ({
    typeIdx: index('opportunities_type_idx').on(table.type),
    profitIdx: index('opportunities_profit_idx').on(table.profitPercentage),
    activeIdx: index('opportunities_active_idx').on(table.isActive),
    expiresIdx: index('opportunities_expires_idx').on(table.expiresAt),
  })
);

export const tradingStrategies = sqliteTable(
  'trading_strategies',
  {
    id: integer('id', { mode: 'number' }).primaryKey({ autoIncrement: true }),
    userId: integer('user_id')
      .notNull()
      .references(() => users.id, { onDelete: 'cascade' }),
    name: text('name').notNull(),
    type: text('type', { enum: ['arbitrage', 'technical', 'manual'] }).notNull(),
    isActive: integer('is_active', { mode: 'boolean' }).notNull().default(true),
    settings: text('settings', { mode: 'json' }).notNull(),
    
    // Performance metrics as JSON field
    performance: text('performance', { mode: 'json' })
      .$type<{
        totalTrades?: number;
        winRate?: number;
        averageReturn?: number;
        maxDrawdown?: number;
        sharpeRatio?: number;
        lastUpdated?: number;
      }>()
      .default(sql`'{}'`),
    
    createdAt: integer('created_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
    updatedAt: integer('updated_at', { mode: 'timestamp' })
      .notNull()
      .default(sql`(unixepoch())`),
  },
  (table) => ({
    userIdIdx: index('trading_strategies_user_id_idx').on(table.userId),
    typeIdx: index('trading_strategies_type_idx').on(table.type),
    activeIdx: index('trading_strategies_active_idx').on(table.isActive),
  })
);

// Export types
export type Position = typeof positions.$inferSelect;
export type NewPosition = typeof positions.$inferInsert;
export type Opportunity = typeof opportunities.$inferSelect;
export type NewOpportunity = typeof opportunities.$inferInsert;
export type TradingStrategy = typeof tradingStrategies.$inferSelect;
export type NewTradingStrategy = typeof tradingStrategies.$inferInsert;