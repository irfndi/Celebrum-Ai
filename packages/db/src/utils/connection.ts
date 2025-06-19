import { drizzle } from 'drizzle-orm/d1';
import type { DrizzleD1Database } from 'drizzle-orm/d1';
import * as schema from '../schema/index.js';

// Type for our database instance
export type Database = DrizzleD1Database<typeof schema>;

// Cloudflare Worker environment interface
export interface Env {
  ArbEdgeD1: D1Database;
}

/**
 * Create database connection for Cloudflare Worker
 * @param d1Database - D1Database instance from Cloudflare Worker env
 * @returns Drizzle database instance with schema
 */
export function createDb(d1Database: D1Database): Database {
  return drizzle(d1Database, { schema });
}

/**
 * Database connection factory for different environments
 * @param env - Cloudflare Worker environment
 * @returns Database instance
 */
export function getDatabase(env: Env): Database {
  return createDb(env.ArbEdgeD1);
}

/**
 * Type-safe database transaction wrapper
 * @param db - Database instance
 * @param fn - Transaction function
 * @returns Promise with transaction result
 */
export async function withTransaction<T>(
  db: Database,
  fn: (tx: Parameters<Parameters<Database['transaction']>[0]>[0]) => Promise<T>
): Promise<T> {
  return await db.transaction(fn);
} 