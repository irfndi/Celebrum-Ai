import { eq, and, desc, sql } from 'drizzle-orm';
import type { Database } from './connection.js';
import { users, positions, opportunities, tradingStrategies } from '../schema/index.js';
import type { 
  User, 
  NewUser, 
  Position, 
  NewPosition, 
  Opportunity,
  NewOpportunity,
  TradingStrategy,
  NewTradingStrategy 
} from '../schema/index.js';

// User operations
export class UserQueries {
  constructor(private db: Database) {}

  async findByTelegramId(telegramId: string): Promise<User | undefined> {
    const result = await this.db
      .select()
      .from(users)
      .where(eq(users.telegramId, telegramId))
      .limit(1);
    return result[0];
  }

  async findById(id: number): Promise<User | undefined> {
    const result = await this.db
      .select()
      .from(users)
      .where(eq(users.id, id))
      .limit(1);
    return result[0];
  }

  async create(user: NewUser): Promise<User> {
    const result = await this.db
      .insert(users)
      .values(user)
      .returning();
    return result[0]!;
  }

  async update(id: number, updates: Partial<NewUser>): Promise<User | undefined> {
    const result = await this.db
      .update(users)
      .set({
        ...updates,
        updatedAt: sql`(unixepoch())`,
      })
      .where(eq(users.id, id))
      .returning();
    return result[0];
  }

  async delete(id: number): Promise<boolean> {
    const result = await this.db
      .delete(users)
      .where(eq(users.id, id));
    
    // D1Result doesn't have changes property, use success flag instead
    return result.success;
  }
}

// Position operations
export class PositionQueries {
  constructor(private db: Database) {}

  async findByUserId(userId: number, status?: 'open' | 'closed' | 'partially_filled' | 'cancelled'): Promise<Position[]> {
    let query = this.db.select().from(positions);
    
    if (status) {
      return await query.where(and(eq(positions.userId, userId), eq(positions.status, status)));
    }
    
    return await query.where(eq(positions.userId, userId));
  }

  async findById(id: number): Promise<Position | undefined> {
    const result = await this.db
      .select()
      .from(positions)
      .where(eq(positions.id, id))
      .limit(1);
    return result[0];
  }

  async create(position: NewPosition): Promise<Position> {
    const result = await this.db
      .insert(positions)
      .values(position)
      .returning();
    return result[0]!;
  }

  async update(id: number, updates: Partial<NewPosition>): Promise<Position | undefined> {
    const result = await this.db
      .update(positions)
      .set({
        ...updates,
        updatedAt: sql`(unixepoch())`,
      })
      .where(eq(positions.id, id))
      .returning();
    return result[0];
  }

  async closePosition(id: number, exitPrice: number, pnl: number): Promise<Position | undefined> {
    const result = await this.db
      .update(positions)
      .set({
        status: 'closed',
        exitPrice,
        pnl,
        closedAt: sql`(unixepoch())`,
        updatedAt: sql`(unixepoch())`,
      })
      .where(eq(positions.id, id))
      .returning();
    return result[0];
  }
}

// Opportunity operations
export class OpportunityQueries {
  constructor(private db: Database) {}

  async findActive(type?: 'arbitrage' | 'technical'): Promise<Opportunity[]> {
    let baseQuery = this.db
      .select()
      .from(opportunities)
      .orderBy(desc(opportunities.profitPercentage));
    
    if (type) {
      return await baseQuery.where(and(
        eq(opportunities.isActive, true),
        eq(opportunities.type, type)
      ));
    }
    
    return await baseQuery.where(eq(opportunities.isActive, true));
  }

  async findById(id: number): Promise<Opportunity | undefined> {
    const result = await this.db
      .select()
      .from(opportunities)
      .where(eq(opportunities.id, id))
      .limit(1);
    return result[0];
  }

  async create(opportunity: NewOpportunity): Promise<Opportunity> {
    const result = await this.db
      .insert(opportunities)
      .values(opportunity)
      .returning();
    return result[0]!;
  }

  async deactivate(id: number): Promise<Opportunity | undefined> {
    const result = await this.db
      .update(opportunities)
      .set({ isActive: false })
      .where(eq(opportunities.id, id))
      .returning();
    return result[0];
  }

  async cleanup(): Promise<number> {
    const result = await this.db
      .delete(opportunities)
      .where(sql`expires_at < unixepoch()`);
    
    // D1Result uses meta.changes for affected rows count
    return result.meta.changes ?? 0;
  }
}

// Trading Strategy operations
export class TradingStrategyQueries {
  constructor(private db: Database) {}

  async findByUserId(userId: number, isActive?: boolean): Promise<TradingStrategy[]> {
    let query = this.db.select().from(tradingStrategies);
    
    if (isActive !== undefined) {
      return await query.where(and(
        eq(tradingStrategies.userId, userId),
        eq(tradingStrategies.isActive, isActive)
      ));
    }
    
    return await query.where(eq(tradingStrategies.userId, userId));
  }

  async findById(id: number): Promise<TradingStrategy | undefined> {
    const result = await this.db
      .select()
      .from(tradingStrategies)
      .where(eq(tradingStrategies.id, id))
      .limit(1);
    return result[0];
  }

  async create(strategy: NewTradingStrategy): Promise<TradingStrategy> {
    const result = await this.db
      .insert(tradingStrategies)
      .values(strategy)
      .returning();
    return result[0]!;
  }

  async updatePerformance(id: number, performance: Record<string, unknown>): Promise<TradingStrategy | undefined> {
    const result = await this.db
      .update(tradingStrategies)
      .set({
        performance: performance,
        updatedAt: sql`(unixepoch())`,
      })
      .where(eq(tradingStrategies.id, id))
      .returning();
    return result[0];
  }

  async toggle(id: number): Promise<TradingStrategy | undefined> {
    // First get the current state
    const current = await this.findById(id);
    if (!current) return undefined;
    
    const result = await this.db
      .update(tradingStrategies)
      .set({
        isActive: !current.isActive,
        updatedAt: sql`(unixepoch())`,
      })
      .where(eq(tradingStrategies.id, id))
      .returning();
    return result[0];
  }
}

// Main query class that combines all operations
export class DatabaseQueries {
  public users: UserQueries;
  public positions: PositionQueries;
  public opportunities: OpportunityQueries;
  public strategies: TradingStrategyQueries;

  constructor(db: Database) {
    this.users = new UserQueries(db);
    this.positions = new PositionQueries(db);
    this.opportunities = new OpportunityQueries(db);
    this.strategies = new TradingStrategyQueries(db);
  }
} 