import { z } from 'zod';
import { CRYPTOCURRENCIES, FIAT_CURRENCIES, EXCHANGE_INFO } from '../constants';

// Base validation schemas
export const emailSchema = z.string().email('Invalid email format');
export const passwordSchema = z.string()
  .min(8, 'Password must be at least 8 characters')
  .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/, 
    'Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character');

export const usernameSchema = z.string()
  .min(3, 'Username must be at least 3 characters')
  .max(30, 'Username must not exceed 30 characters')
  .regex(/^[a-zA-Z0-9_-]+$/, 'Username can only contain letters, numbers, underscores, and hyphens');

export const phoneSchema = z.string()
  .regex(/^\+?[1-9]\d{1,14}$/, 'Invalid phone number format');

// Trading validation schemas
export const symbolSchema = z.string()
  .min(3, 'Symbol must be at least 3 characters')
  .max(20, 'Symbol must not exceed 20 characters')
  .regex(/^[A-Z0-9]+\/[A-Z0-9]+$/, 'Symbol must be in format BASE/QUOTE (e.g., BTC/USDT)');

export const priceSchema = z.number()
  .positive('Price must be positive')
  .finite('Price must be a finite number')
  .refine(val => val > 0.00000001, 'Price too small')
  .refine(val => val < 1000000000, 'Price too large');

export const quantitySchema = z.number()
  .positive('Quantity must be positive')
  .finite('Quantity must be a finite number')
  .refine(val => val > 0.00000001, 'Quantity too small');

export const leverageSchema = z.number()
  .min(1, 'Leverage must be at least 1x')
  .max(100, 'Leverage cannot exceed 100x')
  .int('Leverage must be a whole number');

export const percentageSchema = z.number()
  .min(0, 'Percentage cannot be negative')
  .max(100, 'Percentage cannot exceed 100');

export const exchangeIdSchema = z.enum(['binance', 'bybit', 'okx', 'bitget', 'kucoin'] as const);

// User validation schemas
export const userRoleSchema = z.enum(['admin', 'premium', 'basic'] as const);
export const userStatusSchema = z.enum(['active', 'inactive', 'suspended', 'pending'] as const);

export const createUserSchema = z.object({
  email: emailSchema,
  username: usernameSchema,
  password: passwordSchema,
  firstName: z.string().min(1, 'First name is required').max(50, 'First name too long'),
  lastName: z.string().min(1, 'Last name is required').max(50, 'Last name too long'),
  phone: phoneSchema.optional(),
  role: userRoleSchema.default('basic'),
  status: userStatusSchema.default('pending'),
});

export const updateUserSchema = createUserSchema.partial().omit({ password: true });

export const loginSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, 'Password is required'),
});

export const changePasswordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: passwordSchema,
  confirmPassword: z.string(),
}).refine(data => data.newPassword === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

// Trading validation schemas
export const positionTypeSchema = z.enum(['long', 'short'] as const);
export const positionStatusSchema = z.enum(['open', 'closed', 'liquidated'] as const);
export const opportunityTypeSchema = z.enum(['arbitrage', 'funding', 'spread'] as const);

export const createPositionSchema = z.object({
  userId: z.string().uuid('Invalid user ID'),
  symbol: symbolSchema,
  type: positionTypeSchema,
  size: quantitySchema,
  entryPrice: priceSchema,
  leverage: leverageSchema.default(1),
  stopLoss: priceSchema.optional(),
  takeProfit: priceSchema.optional(),
  exchangeId: exchangeIdSchema,
});

export const updatePositionSchema = z.object({
  exitPrice: priceSchema.optional(),
  status: positionStatusSchema.optional(),
  stopLoss: priceSchema.optional(),
  takeProfit: priceSchema.optional(),
  metadata: z.record(z.unknown()).optional(),
});

export const createOpportunitySchema = z.object({
  type: opportunityTypeSchema,
  symbol: symbolSchema,
  buyExchange: exchangeIdSchema,
  sellExchange: exchangeIdSchema,
  buyPrice: priceSchema,
  sellPrice: priceSchema,
  volume: quantitySchema,
  profitPercentage: percentageSchema,
  confidence: z.number().min(0).max(1),
  expiresAt: z.date(),
});

// API validation schemas
export const paginationSchema = z.object({
  page: z.number().int().min(1).default(1),
  limit: z.number().int().min(1).max(100).default(20),
  sortBy: z.string().optional(),
  sortOrder: z.enum(['asc', 'desc']).default('desc'),
});

export const dateRangeSchema = z.object({
  startDate: z.date(),
  endDate: z.date(),
}).refine(data => data.startDate <= data.endDate, {
  message: 'Start date must be before or equal to end date',
  path: ['endDate'],
});

export const filterSchema = z.object({
  status: z.string().optional(),
  type: z.string().optional(),
  exchange: exchangeIdSchema.optional(),
  symbol: symbolSchema.optional(),
  minAmount: z.number().positive().optional(),
  maxAmount: z.number().positive().optional(),
  startDate: z.date().optional(),
  endDate: z.date().optional(),
});

// Telegram validation schemas
export const telegramUserSchema = z.object({
  id: z.number().int().positive(),
  first_name: z.string(),
  last_name: z.string().optional(),
  username: z.string().optional(),
  language_code: z.string().optional(),
});

export const telegramMessageSchema = z.object({
  message_id: z.number().int().positive(),
  from: telegramUserSchema,
  chat: z.object({
    id: z.number().int(),
    type: z.enum(['private', 'group', 'supergroup', 'channel']),
    title: z.string().optional(),
    username: z.string().optional(),
  }),
  date: z.number().int().positive(),
  text: z.string().optional(),
});

// Configuration validation schemas
export const apiConfigSchema = z.object({
  port: z.number().int().min(1).max(65535),
  host: z.string().min(1),
  cors: z.object({
    origin: z.union([z.string(), z.array(z.string()), z.boolean()]),
    credentials: z.boolean(),
  }),
  rateLimit: z.object({
    windowMs: z.number().int().positive(),
    max: z.number().int().positive(),
  }),
});

export const databaseConfigSchema = z.object({
  url: z.string().url(),
  maxConnections: z.number().int().positive(),
  connectionTimeout: z.number().int().positive(),
  queryTimeout: z.number().int().positive(),
  retryAttempts: z.number().int().min(0),
});

// Utility validation functions
export const validateSymbol = (symbol: string): boolean => {
  try {
    symbolSchema.parse(symbol);
    return true;
  } catch {
    return false;
  }
};

export const validatePrice = (price: number): boolean => {
  try {
    priceSchema.parse(price);
    return true;
  } catch {
    return false;
  }
};

export const validateQuantity = (quantity: number): boolean => {
  try {
    quantitySchema.parse(quantity);
    return true;
  } catch {
    return false;
  }
};

export const validateExchange = (exchange: string): exchange is keyof typeof EXCHANGE_INFO => {
  return Object.keys(EXCHANGE_INFO).includes(exchange);
};

export const validateCryptocurrency = (crypto: string): boolean => {
  return Object.keys(CRYPTOCURRENCIES).includes(crypto.toUpperCase());
};

export const validateFiatCurrency = (fiat: string): boolean => {
  return Object.keys(FIAT_CURRENCIES).includes(fiat.toUpperCase());
};

// Custom validation helpers
export const createEnumSchema = <T extends readonly [string, ...string[]]>(values: T, name: string) => {
  return z.enum(values, {
    errorMap: () => ({ message: `Invalid ${name}. Must be one of: ${values.join(', ')}` }),
  });
};

export const createOptionalStringSchema = (minLength = 0, maxLength = 255) => {
  return z.string().min(minLength).max(maxLength).optional().or(z.literal(''));
};

export const createRequiredStringSchema = (minLength = 1, maxLength = 255) => {
  return z.string().min(minLength, `Must be at least ${minLength} characters`)
    .max(maxLength, `Must not exceed ${maxLength} characters`);
};

// Validation error formatter
export const formatValidationErrors = (error: z.ZodError) => {
  return error.errors.map(err => ({
    field: err.path.join('.'),
    message: err.message,
    code: err.code,
  }));
};

// Safe validation wrapper
export const safeValidate = <T>(schema: z.ZodSchema<T>, data: unknown) => {
  const result = schema.safeParse(data);
  if (result.success) {
    return { success: true, data: result.data, errors: null };
  }
  return {
    success: false,
    data: null,
    errors: formatValidationErrors(result.error),
  };
};

// Validation middleware helper
export const createValidationMiddleware = <T>(schema: z.ZodSchema<T>) => {
  return (data: unknown) => {
    const result = safeValidate(schema, data);
    if (!result.success) {
      throw new Error(`Validation failed: ${JSON.stringify(result.errors)}`);
    }
    return result.data;
  };
};

// Export all schemas for external use
export const schemas = {
  // Base schemas
  email: emailSchema,
  password: passwordSchema,
  username: usernameSchema,
  phone: phoneSchema,
  
  // Trading schemas
  symbol: symbolSchema,
  price: priceSchema,
  quantity: quantitySchema,
  leverage: leverageSchema,
  percentage: percentageSchema,
  exchangeId: exchangeIdSchema,
  
  // User schemas
  userRole: userRoleSchema,
  userStatus: userStatusSchema,
  createUser: createUserSchema,
  updateUser: updateUserSchema,
  login: loginSchema,
  changePassword: changePasswordSchema,
  
  // Trading schemas
  positionType: positionTypeSchema,
  positionStatus: positionStatusSchema,
  opportunityType: opportunityTypeSchema,
  createPosition: createPositionSchema,
  updatePosition: updatePositionSchema,
  createOpportunity: createOpportunitySchema,
  
  // API schemas
  pagination: paginationSchema,
  dateRange: dateRangeSchema,
  filter: filterSchema,
  
  // Telegram schemas
  telegramUser: telegramUserSchema,
  telegramMessage: telegramMessageSchema,
  
  // Config schemas
  apiConfig: apiConfigSchema,
  databaseConfig: databaseConfigSchema,
};