import { vi } from 'vitest';
// Import the factory and type from the helpers file
import { 
  createMockInstance, 
  type MockExchangeInstance, 
  ALL_MOCK_EXCHANGE_IDS, 
  _testMockInstances
} from '../services/exchangeService.test.helpers';
import type { ExchangeId } from '../../src/types'; // Import ExchangeId for casting

interface MockLimit {
  min?: number;
  max?: number;
}

interface MockLimits {
  amount: MockLimit;
  price: MockLimit;
  cost?: MockLimit;
  leverage?: MockLimit;
}

interface MockPrecision {
  amount?: number;
  price?: number;
  base?: number;
  quote?: number;
}

interface MockMarket {
  symbol: string;
  base: string;
  quote: string;
  active: boolean;
  type: 'spot' | 'swap' | 'future' | 'option'; // Add other relevant types
  taker: number;
  maker: number;
  limits: MockLimits;
  precision: MockPrecision;
  linear?: boolean;
  inverse?: boolean;
  settle?: string;
  // Add other fields as necessary from CCXT Market type or usage
  id?: string;
  lowercaseId?: string;
  baseId?: string;
  quoteId?: string;
  settleId?: string;
  percentage?: boolean;
  tierBased?: boolean;
  spot?: boolean;
  margin?: boolean;
  swap?: boolean;
  future?: boolean;
  option?: boolean;
  contract?: boolean;
  contractSize?: number;
  expiry?: number;
  expiryDatetime?: string;
  optionType?: 'call' | 'put';
  strike?: number;
  info?: unknown; // Keep info as unknown for flexibility from real CCXT (Linter fix: any -> unknown)
}

// Default mock for any unhandled methods
const unhandledMethod = (methodName: string) => vi.fn().mockImplementation(async (...args: unknown[]) => {
  console.warn(`CCXT MOCK WARN: Unhandled method call: ${methodName}`, args);
  const lowerMethodName = methodName.toLowerCase();
  if (lowerMethodName.includes('fetch')) {
    if (lowerMethodName.includes('ohlcv')) return [];
    if (lowerMethodName.includes('orderbook')) return { bids: [], asks: [], timestamp: Date.now(), datetime: new Date().toISOString(), nonce: undefined };
    if (lowerMethodName.endsWith('s') || lowerMethodName.includes('trades') || lowerMethodName.includes('orders') || lowerMethodName.includes('positions')) {
        return [];
    }
    return {};
  }
  if (lowerMethodName.startsWith('create') || lowerMethodName.startsWith('edit') || lowerMethodName.startsWith('cancel')) {
    return { info: {}, id: 'mock-id', timestamp: Date.now(), datetime: new Date().toISOString() };
  }
  if (lowerMethodName.startsWith('set') || lowerMethodName.startsWith('transfer')) {
    return { info: {} };
  }
  return Promise.resolve({});
});

const mockExchangeInstance = {
  // Properties
  rateLimit: 1000, // Default rateLimit
  enableRateLimit: true,
  timeout: 30000,
  verbose: false,
  markets: {} as Record<string, MockMarket>,
  currencies: {},
  ids: [],
  options: {},
  fees: { trading: {}, funding: {} },
  userAgent: {},
  headers: {},


  // Core methods used by ExchangeService
  loadMarkets: vi.fn().mockImplementation(async () => {
    console.log('CCXT MOCK: loadMarkets called');
    // Simulate loading markets for a few common symbols
    mockExchangeInstance.markets = {
      'BTC/USDT': { symbol: 'BTC/USDT', base: 'BTC', quote: 'USDT', active: true, type: 'spot', taker: 0.001, maker: 0.001, limits: { amount: { min: 0.0001, max: 1000 }, price: { min: 0.01, max: 1000000 }, cost: { min: 10 } }, precision: { amount: 4, price: 2 } },
      'ETH/USDT': { symbol: 'ETH/USDT', base: 'ETH', quote: 'USDT', active: true, type: 'spot', taker: 0.001, maker: 0.001, limits: { amount: { min: 0.001, max: 10000 }, price: { min: 0.01, max: 100000 }, cost: { min: 10 } }, precision: { amount: 3, price: 2 } },
      'SOL/USDT:USDT': { symbol: 'SOL/USDT:USDT', base: 'SOL', quote: 'USDT', active: true, type: 'swap', linear: true, taker: 0.0005, maker: 0.0002, limits: { amount: { min: 0.1, max: 100000 }, price: { min: 0.01, max: 100000 }, cost: { min: 1 } }, precision: { amount: 1, price: 3 }, settle: 'USDT' },
    };
    return mockExchangeInstance.markets;
  }),
  fetchTicker: vi.fn().mockImplementation(async (symbol: string) => {
    console.log(`CCXT MOCK: fetchTicker called for ${symbol}`);
    if (symbol === 'BTC/USDT') {
      return { symbol, timestamp: Date.now(), datetime: new Date().toISOString(), high: 30000, low: 29000, bid: 29500, ask: 29505, last: 29502, close: 29502, open: 29400, previousClose: 29400, change: 102, percentage: 0.34, average: 29451, vwap: 29500, baseVolume: 1000, quoteVolume: 29500000, info: {} };
    }
    return { symbol, timestamp: Date.now(), datetime: new Date().toISOString(), bid: 0, ask: 0, last: 0, info: {} };
  }),
  fetchTickers: vi.fn().mockImplementation(async (symbols?: string[]) => {
    console.log(`CCXT MOCK: fetchTickers called for ${symbols?.join(', ')}`);
    const tickers: Record<string, unknown> = {}; // Changed any to unknown
    if (symbols) {
      for (const s of symbols) { // Changed forEach to for...of
        tickers[s] = { symbol: s, timestamp: Date.now(), datetime: new Date().toISOString(), bid: 0, ask: 0, last: 0, info: {} };
      }
    } else { // Mock a couple if no specific symbols requested
      tickers['BTC/USDT'] = { symbol: 'BTC/USDT', timestamp: Date.now(), datetime: new Date().toISOString(), bid: 29500, ask: 29505, last: 29502, info: {} };
      tickers['ETH/USDT'] = { symbol: 'ETH/USDT', timestamp: Date.now(), datetime: new Date().toISOString(), bid: 1800, ask: 1801, last: 1800.5, info: {} };
    }
    return tickers;
  }),
  createOrder: vi.fn().mockImplementation(async (symbol, type, side, amount, price, params: Record<string, unknown> = {}) => { // Added type for params
    console.log(`CCXT MOCK: createOrder called for ${symbol}, ${type}, ${side}, ${amount}, ${price}`);
    return { id: `mock-order-id-${Date.now()}`, clientOrderId: (params?.clientOrderId as string | undefined) || `mock-client-order-id-${Date.now()}`, timestamp: Date.now(), datetime: new Date().toISOString(), lastTradeTimestamp: undefined, symbol, type, timeInForce: 'GTC', postOnly: false, reduceOnly: false, side, price, amount, cost: price ? amount * price : amount, average: price, filled: 0, remaining: amount, status: 'open', fee: undefined, trades: [], info: {} }; // Changed to template literals and typed clientOrderId access
  }),
  fetchBalance: vi.fn().mockImplementation(async () => {
    console.log('CCXT MOCK: fetchBalance called');
    return {
      info: {},
      free: { USDT: 10000, BTC: 1, ETH: 10 },
      used: { USDT: 100, BTC: 0.1, ETH: 1 },
      total: { USDT: 10100, BTC: 1.1, ETH: 11 },
    };
  }),
  setLeverage: vi.fn().mockImplementation(async (leverage, symbol, params = {}) => {
    console.log(`CCXT MOCK: setLeverage called for ${symbol} to ${leverage}`);
    return { info: { symbol, leverage } };
  }),
  fetchOHLCV: vi.fn().mockImplementation(async (symbol, timeframe = '1m', since = undefined, limit = undefined, params = {}) => {
    console.log(`CCXT MOCK: fetchOHLCV called for ${symbol}, timeframe ${timeframe}`);
    // Return a few mock candles
    const now = Date.now();
    return [
      [now - 3 * 60000, 29000, 29100, 28900, 29050, 10], // [timestamp, open, high, low, close, volume]
      [now - 2 * 60000, 29050, 29150, 29000, 29100, 12],
      [now - 1 * 60000, 29100, 29200, 29050, 29150, 15],
    ];
  }),
  fetchFundingRate: vi.fn().mockImplementation(async (symbol: string, params = {}) => {
    console.log(`CCXT MOCK: fetchFundingRate called for ${symbol}`);
    return {
      symbol,
      timestamp: Date.now(),
      datetime: new Date().toISOString(),
      fundingRate: 0.0001,
      markPrice: 29500,
      indexPrice: 29495,
      interestRate: 0, // Or some mock value
      estimatedSettlePrice: 29500,
      fundingTimestamp: Date.now() + 8 * 60 * 60 * 1000, // Mock next funding time
      info: {},
    };
  }),
  fetchFundingRates: vi.fn().mockImplementation(async (symbols?: string[], params = {}) => {
    console.log(`CCXT MOCK: fetchFundingRates called for ${symbols?.join(', ')}`);
    const rates: unknown[] = []; // Changed any[] to unknown[]
    const effectiveSymbols = symbols || ['BTC/USDT:USDT', 'ETH/USDT:USDT'];
    for (const s of effectiveSymbols) { // Changed forEach to for...of
      rates.push({
        symbol: s,
        timestamp: Date.now(),
        datetime: new Date().toISOString(),
        fundingRate: Math.random() * 0.0002 - 0.0001, // Random small rate
        markPrice: Math.random() * 50000,
        indexPrice: Math.random() * 50000,
        info: {},
      });
    }
    return rates;
  }),
  cancelOrder: vi.fn().mockImplementation(async (id: string, symbol?: string, params = {}) => {
    console.log(`CCXT MOCK: cancelOrder called for ID ${id}, symbol ${symbol}`);
    return { id, symbol, status: 'canceled', info: {} };
  }),
  fetchOpenOrders: vi.fn().mockImplementation(async (symbol?: string, since?: number, limit?: number, params = {}) => {
    console.log(`CCXT MOCK: fetchOpenOrders called for symbol ${symbol}`);
    return []; // Default to no open orders
  }),
  // fetchPositions can be tricky as its signature varies (some exchanges have fetchPositions, some fetchPosition)
  // For now, provide a generic mock. ExchangeService seems to check for its existence.
  fetchPositions: vi.fn().mockImplementation(async (symbols?: string[], params = {}) => {
    console.log(`CCXT MOCK: fetchPositions called for symbols ${symbols?.join(', ')}`);
    if (symbols && symbols.length > 0) {
      return symbols.map(s => ({
        symbol: s,
        side: (Math.random() > 0.5 ? 'long' : 'short'),
        contracts: Math.random() * 10,
        entryPrice: Math.random() * 30000,
        markPrice: Math.random() * 30000,
        unrealizedPnl: Math.random() * 100 - 50,
        leverage: 10,
        collateral: Math.random() * 3000,
        info: {},
      }));
    }
    return []; // Default to no positions
  }),
  // Add other methods used by ExchangeService as vi.fn() initially
  // e.g., editOrder, fetchOrder, fetchMyTrades, etc., if they become necessary
  has: { // Mock the 'has' object for feature detection if ExchangeService uses it
    'fetchFundingRates': true,
    'fetchPositions': true,
    'CORS': false,
    // Add other features as needed by tests
  },

  // Fallback for any other properties or methods
  // This uses a Proxy to dynamically create mock functions for unhandled method calls
  // Note: Proxies might have limitations or performance implications in some environments.
  // If issues arise, explicit mocks for all used methods are safer.
  // ... unhandledMethod('default') // This was a placeholder, a Proxy is more comprehensive.
  // Removed Proxy as it was not fully implemented and explicit mocks per method are preferred for clarity in ccxt.
};

// This will store the uniquely created instances for test access, defined independently
export const _testAccessibleMockInstances = _testMockInstances;

// Common CCXT error classes - create proper error classes with inheritance
export class BaseError extends Error {
  constructor(message: string) {
    super(message);
    this.name = this.constructor.name;
    // Ensure message is properly set and accessible
    if (message) this.message = message;
    
    // Ensure stack trace is captured
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }

  toString() {
    return `${this.name}: ${this.message}`;
  }
}

export class NetworkError extends BaseError {
  constructor(message = "") {
    super(message);
  }
}

export class ExchangeError extends BaseError {
  constructor(message = "") {
    super(message);
  }
}

export class AuthenticationError extends BaseError {
  constructor(message = "Invalid API key or insufficient permissions.") {
    super(message);
  }
}

export class PermissionDenied extends BaseError {
  constructor(message = "Access to the resource is restricted.") {
    super(message);
  }
}

export class InvalidNonce extends BaseError {
  constructor(message = "Invalid nonce or timestamp. Please try again.") {
    super(message);
  }
}

export class OrderNotFound extends BaseError {
  constructor(message = "Order not found") {
    super(message);
  }
}

export class InsufficientFunds extends BaseError {
  constructor(message = "Insufficient funds to perform the operation.") {
    super(message);
  }
}

export class InvalidOrder extends BaseError {
  constructor(message = "The order is invalid or not acceptable by the exchange.") {
    super(message);
  }
}

export class RateLimitExceeded extends BaseError {
  constructor(message = "Rate limit exceeded. Please try again later.") {
    super(message);
  }
}

export class DDoSProtection extends BaseError {
  constructor(message = "DDoS protection activated") {
    super(message);
  }
}

export class RequestTimeout extends BaseError {
  constructor(message = "Request timed out") {
    super(message);
  }
}

const commonErrorClasses = {
  NetworkError,
  ExchangeError,
  AuthenticationError,
  PermissionDenied,
  InvalidNonce,
  OrderNotFound,
  InsufficientFunds,
  InvalidOrder,
  RateLimitExceeded,
  DDoSProtection,
  RequestTimeout
};

// Helper function to create exchange mock constructors
// This function closes over _testAccessibleMockInstances
const createExchangeMockConstructor = (exchangeId: ExchangeId, specificInstanceCreator: () => MockExchangeInstance) => {
  return vi.fn().mockImplementation((config?: Record<string, unknown>) => {
    console.log(`CCXT MOCK: constructor for ${exchangeId} called with config:`, config);
    const instance = specificInstanceCreator();
    // Populate the central accessible instances record
    _testAccessibleMockInstances[exchangeId] = instance;
    console.log(`CCXT MOCK: instance for ${exchangeId} created and stored. Config:`, config, "Instance keys:", Object.keys(instance));
    return instance;
  });
};

// Explicitly defined exchanges using the helper
const binance = createExchangeMockConstructor('binance' as ExchangeId, () => createMockInstance('binance'));
const binanceusdm = createExchangeMockConstructor('binanceusdm' as ExchangeId, () => createMockInstance('binanceusdm' as ExchangeId));
const bybit = createExchangeMockConstructor('bybit' as ExchangeId, () => createMockInstance('bybit' as ExchangeId));
const kraken = createExchangeMockConstructor('kraken' as ExchangeId, () => createMockInstance('kraken' as ExchangeId));
const coinbase = createExchangeMockConstructor('coinbase' as ExchangeId, () => createMockInstance('coinbase' as ExchangeId));
const okx = createExchangeMockConstructor('okx' as ExchangeId, () => createMockInstance('okx'));
const gateio = createExchangeMockConstructor('gateio' as ExchangeId, () => createMockInstance('gateio'as ExchangeId));
const kucoin = createExchangeMockConstructor('kucoin' as ExchangeId, () => createMockInstance('kucoin'as ExchangeId));
const bingx = createExchangeMockConstructor('bingx' as ExchangeId, () => createMockInstance('bingx'as ExchangeId));
const bitget = createExchangeMockConstructor('bitget' as ExchangeId, () => createMockInstance('bitget'as ExchangeId));
const mexc = createExchangeMockConstructor('mexc' as ExchangeId, () => createMockInstance('mexc'as ExchangeId));
// Add other known exchanges here using createExchangeMockConstructor if desired for explicitness

// Object containing all exchange mock constructors
export const mockExchangeConstructors = {
  binance,
  binanceusdm,
  bybit,
  kraken,
  coinbase,
  okx,
  gateio,
  kucoin,
  bingx,
  bitget,
  mexc,
  // Add any other explicitly defined exchange mocks here
};

// Add metadata
export const version = '3.1.mock';  // Mocked CCXT version

// Create the default export for the module
const defaultExport = {
  version,
  // Include the exchange constructors
  ...mockExchangeConstructors,
  // Common error classes available via CCXT module import
  ...commonErrorClasses,
  // Keep track of specific instances for tests to access 
  _testAccessibleMockInstances
};

export default defaultExport;

export function Exchange(config?: Record<string, unknown>): MockExchangeInstance {
  return createMockInstance('binance');
} 