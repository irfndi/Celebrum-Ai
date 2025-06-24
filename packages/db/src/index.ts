// Main database exports
export * from './schema/index';
export * from './utils/connection';
export * from './utils/queries';

// Re-export commonly used Drizzle types and functions
export { eq, and, or, not, sql, desc, asc, like, ilike } from 'drizzle-orm';
export type { SQL } from 'drizzle-orm';

// Package version
export const VERSION = '0.1.0';