# @celebrum-ai/shared

Shared types, utilities, configurations, and middleware for ArbEdge services.

## Overview

This package provides common functionality used across all ArbEdge services including the worker, telegram-bot, web, and database packages. It ensures consistency and reduces code duplication across the monorepo.

## Installation

```bash
npm install @celebrum-ai/shared
# or
yarn add @celebrum-ai/shared
# or
pnpm add @celebrum-ai/shared
```

## Modules

### Types

Comprehensive TypeScript types and Zod schemas for the entire application.

```typescript
import { User, Position, Opportunity, ApiResponse } from '@celebrum-ai/shared/types';

// User types
const user: User = {
  id: '123',
  email: 'user@example.com',
  username: 'trader123',
  role: 'premium',
  status: 'active',
  // ... other fields
};

// Trading types
const position: Position = {
  id: '456',
  userId: '123',
  symbol: 'BTC/USDT',
  type: 'long',
  size: 1.5,
  entryPrice: 45000,
  // ... other fields
};
```

### Utils

Utility functions for common operations across services.

```typescript
import { 
  formatNumber, 
  calculateProfitPercentage, 
  validateSymbol,
  addMinutes,
  calculatePositionSize 
} from '@celebrum-ai/shared/utils';

// Formatting
const formatted = formatNumber(1234567.89); // "1,234,567.89"
const large = formatLargeNumber(1500000); // "1.5M"

// Trading calculations
const profit = calculateProfitPercentage(100, 110); // 10
const pnl = calculatePnL(1.5, 45000, 46000, 'long'); // 1500

// Validation
const isValid = validateSymbol('BTC/USDT'); // true
const validPrice = validatePrice(45000.50); // true

// Time utilities
const futureTime = addMinutes(new Date(), 30);
const timeLeft = getTimeUntilExpiry(futureTime);

// Risk management
const size = calculatePositionSize(10000, 0.02, 100); // 2
```

### Config

Centralized configuration constants and environment-specific settings.

```typescript
import { 
  API_CONFIG, 
  DB_CONFIG, 
  TRADING_CONFIG,
  ERROR_MESSAGES,
  SUCCESS_MESSAGES,
  validateConfig,
  getConfigForEnvironment 
} from '@celebrum-ai/shared/config';

// API configuration
console.log(API_CONFIG.DEFAULT_PORT); // 3000
console.log(API_CONFIG.RATE_LIMIT.MAX_REQUESTS); // 100

// Trading configuration
console.log(TRADING_CONFIG.MAX_LEVERAGE); // 100
console.log(TRADING_CONFIG.MIN_TRADE_AMOUNT); // 10

// Error messages
console.log(ERROR_MESSAGES.INVALID_CREDENTIALS); // "Invalid email or password"

// Environment-specific config
const config = getConfigForEnvironment('production');
```

### Constants

Application-wide constants including supported exchanges, currencies, and validation patterns.

```typescript
import { 
  SUPPORTED_CRYPTOCURRENCIES,
  SUPPORTED_FIAT_CURRENCIES,
  EXCHANGES,
  TRADING_PAIRS,
  NOTIFICATION_TYPES,
  WEBSOCKET_EVENTS,
  REGEX_PATTERNS 
} from '@celebrum-ai/shared/constants';

// Supported assets
console.log(SUPPORTED_CRYPTOCURRENCIES); // ['BTC', 'ETH', 'USDT', ...]
console.log(SUPPORTED_FIAT_CURRENCIES); // ['USD', 'EUR', 'GBP', ...]

// Exchange information
console.log(EXCHANGES.binance.name); // "Binance"
console.log(EXCHANGES.binance.fees.maker); // 0.001

// Popular trading pairs
console.log(TRADING_PAIRS.major); // ['BTC/USDT', 'ETH/USDT', ...]

// Validation patterns
const emailRegex = REGEX_PATTERNS.email;
const symbolRegex = REGEX_PATTERNS.tradingSymbol;
```

### Errors

Custom error classes and error handling utilities.

```typescript
import { 
  AppError,
  ValidationError,
  AuthenticationError,
  TradingError,
  InsufficientBalanceError,
  handleError,
  formatErrorResponse 
} from '@celebrum-ai/shared/errors';

// Custom errors
throw new ValidationError('Invalid email format', { field: 'email' });
throw new InsufficientBalanceError(1000, 500);
throw new TradingError('Order execution failed');

// Error handling
try {
  // Some operation
} catch (error) {
  const appError = handleError(error);
  const response = formatErrorResponse(appError);
  console.log(response);
}
```

### Validation

Zod schemas and validation utilities for request/response validation.

```typescript
import { 
  schemas,
  validateSymbol,
  validatePrice,
  safeValidate,
  createValidationMiddleware 
} from '@celebrum-ai/shared/validation';

// Schema validation
const userResult = schemas.createUser.safeParse(userData);
if (userResult.success) {
  console.log('Valid user:', userResult.data);
}

// Quick validation
const isValidSymbol = validateSymbol('BTC/USDT'); // true
const isValidPrice = validatePrice(45000.50); // true

// Safe validation with error handling
const result = safeValidate(schemas.createPosition, positionData);
if (result.success) {
  console.log('Valid position:', result.data);
} else {
  console.log('Validation errors:', result.errors);
}

// Middleware creation
const validateUser = createValidationMiddleware(schemas.createUser);
```

### Middleware

Reusable middleware functions for Express.js and similar frameworks.

```typescript
import { 
  middleware,
  corsMiddleware,
  rateLimitMiddleware,
  authMiddleware,
  validateMiddleware,
  errorMiddleware 
} from '@celebrum-ai/shared/middleware';

// Individual middleware
app.use(corsMiddleware());
app.use(rateLimitMiddleware({ max: 100, windowMs: 60000 }));
app.use(authMiddleware({ required: true, roles: ['admin'] }));
app.use(validateMiddleware(schemas.createUser, 'body'));
app.use(errorMiddleware());

// Composed middleware
const apiMiddleware = middleware.compose(
  middleware.cors(),
  middleware.rateLimit(),
  middleware.auth(),
  middleware.logging(),
  middleware.security()
);

app.use('/api', apiMiddleware);
```

## Usage Examples

### Complete API Route Example

```typescript
import express from 'express';
import { 
  middleware,
  schemas,
  AppError,
  formatErrorResponse,
  SUCCESS_MESSAGES 
} from '@celebrum-ai/shared';

const app = express();

// Apply global middleware
app.use(middleware.cors());
app.use(middleware.rateLimit());
app.use(middleware.logging());
app.use(middleware.security());
app.use(express.json());

// Protected route with validation
app.post('/api/positions',
  middleware.auth({ required: true }),
  middleware.validate(schemas.createPosition, 'body'),
  async (req, res, next) => {
    try {
      const position = await createPosition(req.body);
      res.json({
        success: true,
        message: SUCCESS_MESSAGES.POSITION_CREATED,
        data: position
      });
    } catch (error) {
      next(error);
    }
  }
);

// Error handling
app.use(middleware.error());
```

### Database Integration Example

```typescript
import { User, NewUser, schemas } from '@celebrum-ai/shared';
import { db } from '@celebrum-ai/db';

// Validate and create user
export async function createUser(userData: unknown): Promise<User> {
  // Validate input
  const validatedData = schemas.createUser.parse(userData);
  
  // Create user in database
  const [user] = await db.insert(users).values(validatedData).returning();
  
  return user;
}
```

### Trading Service Example

```typescript
import { 
  calculateProfitPercentage,
  validateSymbol,
  TradingError,
  TRADING_CONFIG 
} from '@celebrum-ai/shared';

export async function executeArbitrage(opportunity: Opportunity) {
  // Validate symbol
  if (!validateSymbol(opportunity.symbol)) {
    throw new TradingError('Invalid trading symbol');
  }
  
  // Check minimum profit threshold
  const profit = calculateProfitPercentage(
    opportunity.buyPrice, 
    opportunity.sellPrice
  );
  
  if (profit < TRADING_CONFIG.MIN_PROFIT_THRESHOLD) {
    throw new TradingError('Profit below minimum threshold');
  }
  
  // Execute trades...
}
```

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm run test
npm run test:watch
npm run test:coverage
```

### Linting

```bash
npm run lint
npm run lint:fix
```

### Type Checking

```bash
npm run typecheck
```

## Contributing

When adding new shared functionality:

1. **Types**: Add to `src/types/index.ts` with corresponding Zod schemas
2. **Utils**: Add to `src/utils/index.ts` with proper JSDoc comments
3. **Config**: Add to `src/config/index.ts` with environment validation
4. **Constants**: Add to `src/constants/index.ts` with proper categorization
5. **Errors**: Add custom error classes to `src/errors/index.ts`
6. **Validation**: Add schemas to `src/validation/index.ts`
7. **Middleware**: Add reusable middleware to `src/middleware/index.ts`

Ensure all new exports are added to the main `src/index.ts` file and update the package.json exports if needed.

## License

Private package for ArbEdge project.