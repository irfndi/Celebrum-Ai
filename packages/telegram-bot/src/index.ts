// Main exports for Telegram Bot TypeScript package
export type * from './types/index';
export * from './handlers/index';
export * from './utils/index';

// Re-export commonly used types for convenience
export type { 
  ExecutionContext,
  Request,
  Response 
} from '@cloudflare/workers-types';

// Re-export shared utilities and types
export { 
  AppError,
  ValidationError,
  formatTimestamp,
  isExpired,
  isValidTelegramId,
  isValidEmail,
  UserRole,
  UserStatus,
  APP_NAME,
  APP_VERSION
} from '@celebrum-ai/shared';
export type { Env, ApiResponse } from '@celebrum-ai/shared';

// Package version
export const VERSION = '0.1.0';

// Package name for logging
export const PACKAGE_NAME = '@celebrum-ai/telegram-bot';