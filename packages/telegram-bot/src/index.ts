// Main exports for Telegram Bot TypeScript package
export * from './types/index';
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
  ValidationError 
} from '@arb-edge/shared/errors';
export { 
  formatTimestamp,
  isExpired,
  isValidTelegramId,
  isValidEmail 
} from '@arb-edge/shared/utils';
export type { 
  ApiResponse 
} from '@arb-edge/shared/types';
export { 
  UserRole,
  UserStatus 
} from '@arb-edge/shared/types';
export { 
  APP_NAME,
  APP_VERSION 
} from '@arb-edge/shared/constants';

// Package version
export const VERSION = '0.1.0';

// Package name for logging
export const PACKAGE_NAME = '@arb-edge/telegram-bot';