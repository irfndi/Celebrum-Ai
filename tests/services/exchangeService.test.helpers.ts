/// <reference types='@cloudflare/workers-types' />
/* eslint-disable */
import { vi } from 'vitest';
import type { Mock } from 'vitest';
import type { ExchangeId, TradingPairSymbol, FundingRateInfo, Balances, Position, Order, Market, Ticker, OHLCV, OrderBook, Trade, CCXTTradingFees } from '../../src/types';
import type { Currency as CCXTCurrency, FundingRateHistory as CCXTFundingRateHistory, TransferEntry as CCXTTransferEntry } from 'ccxt';
import { createMockMarket, MOCK_MARKET_DEFAULTS } from '../mocks/marketMocks';
import { mockPositionFactory } from '../mocks/positionMocks';
import { mockTickerFactory } from '../mocks/tickerMocks';
import { mockOrderFactory } from '../mocks/orderMocks';
import { mockTradeFactory } from '../mocks/tradeMocks';
import { MOCK_TRADING_FEES_FACTORY } from '../mocks/feeMocks';
import { MOCK_TIMESTAMP } from '../mocks/mockUtils';
import type { ExchangeService } from '../../src/services/exchangeService';
import type { Logger } from '../../src/utils/logger';

// This interface was in exchangeService.test.ts, moved here as createMockInstance returns it.
export interface MockExchangeInstance { // ccxt.Exchange
  id: string;
  loadMarkets: Mock<() => Promise<Record<string, Market>>>;
  fetchMarkets: Mock<() => Promise<Market[]>>;
  fetchTicker: Mock<(symbol: string) => Promise<Ticker>>;
  fetchTickers: Mock<(symbols?: string[]) => Promise<Record<string, Ticker>>>; 
  fetchOrderBook: Mock<(symbol: string, limit?: number) => Promise<OrderBook>>; 
  fetchOHLCV: Mock<(symbol: string, timeframe?: string, since?: number, limit?: number) => Promise<OHLCV[]>>;
  fetchTrades: Mock<(symbol: string, since?: number, limit?: number) => Promise<Trade[]>>; 
  fetchBalance: Mock<() => Promise<Balances>>; 
  createOrder: Mock<(symbol: string, type: string, side: string, amount: number, price?: number, params?: Record<string, any>) => Promise<Order>>;
  cancelOrder: Mock<(id: string, symbol?: string, params?: Record<string, any>) => Promise<Order>>;
  fetchOrder: Mock<(id: string, symbol?: string, params?: Record<string, any>) => Promise<Order>>;
  fetchOpenOrders: Mock<(symbol?: string, since?: number, limit?: number, params?: Record<string, any>) => Promise<Order[]>>;
  fetchClosedOrders: Mock<(symbol?: string, since?: number, limit?: number, params?: Record<string, any>) => Promise<Order[]>>;
  fetchMyTrades: Mock<(symbol?: string, since?: number, limit?: number, params?: Record<string, any>) => Promise<Trade[]>>;
  fetchPositions: Mock<(symbols?: string[], params?: Record<string, any>) => Promise<Position[]>>;
  fetchFundingRate: Mock<(symbol: string, params?: Record<string, any>) => Promise<FundingRateInfo>>; 
  fetchFundingRates: Mock<(symbols?: string[], params?: Record<string, any>) => Promise<Record<string, FundingRateInfo>>>; 
  fetchFundingRateHistory: Mock<(symbol?: string, since?: number, limit?: number, params?: Record<string, any>) => Promise<CCXTFundingRateHistory[]>>; 
  setLeverage: Mock<(leverage: number, symbol?: string, params?: Record<string, any>) => Promise<any>>; // Using 'any' for Leverage as its def is test-specific
  transfer: Mock<(code: string, amount: number, fromAccount: string, toAccount: string, params?: Record<string, any>) => Promise<CCXTTransferEntry>>; 
  fetchTradingFees: Mock<(params?: Record<string, any>) => Promise<CCXTTradingFees>>; 
  has: Record<string, boolean | string>;
  markets: Record<string, Market>;
  currencies: Record<string, CCXTCurrency>; 
  verbose?: boolean;
  options?: Record<string, any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any; 
}

export function MOCK_BINANCE_BALANCES_FACTORY(): Balances {
	return {
		USDT: { currency: 'USDT', free: 1000, used: 0, total: 1000 },
		BTC: { currency: 'BTC', free: 1, used: 0, total: 1 },
		ETH: { currency: 'ETH', free: 10, used: 0, total: 10 },
	};
}

export function MOCK_FUNDING_RATE_INFO_FACTORY(): FundingRateInfo {
  return {
  symbol: { symbol: 'BTC/USDT:USDT', base: 'BTC', quote: 'USDT', type: 'swap' },
  exchange: 'binance' as ExchangeId,
  fundingRate: 0.0001,
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  markPrice: 20000,
  indexPrice: 19990,
  };
}

export function MOCK_CCXT_FUNDING_RATE_HISTORY_ITEM_FACTORY(): CCXTFundingRateHistory {
  return {
  symbol: 'BTC/USDT:USDT',
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  fundingRate: 0.0001,
  info: {},
  };
}

export function MOCK_CCXT_TRANSFER_ENTRY_FACTORY(): CCXTTransferEntry {
  return {
  id: 'mocktransfer123',
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  currency: 'USDT',
  amount: 100,
  fromAccount: 'spot',
  toAccount: 'futures',
  status: 'ok',
  info: {},
  };
}

export function createMockInstance(exchangeId: ExchangeId | string): MockExchangeInstance {
  const pair: TradingPairSymbol = 'BTC/USDT';
  const base: string = 'BTC';
  const quote: string = 'USDT';
  const marketDefaultsForPair: Market = { ...MOCK_MARKET_DEFAULTS, symbol: pair, base, quote } as Market;
  const mockMarketForInstance: Market = createMockMarket({ created: MOCK_TIMESTAMP, symbol: pair, base, quote });
  const mockBalance = MOCK_BINANCE_BALANCES_FACTORY(); 
  const mockPositions = [mockPositionFactory({ symbol: 'BTC/USDT' as TradingPairSymbol, side: 'long' })];
  const symbolForFundingRatesKey = mockMarketForInstance?.symbol ?? 'BTC/USDT';

  return {
    id: exchangeId,
    loadMarkets: vi.fn().mockResolvedValue({ 'BTC/USDT': mockMarketForInstance }),
    fetchMarkets: vi.fn().mockResolvedValue([mockMarketForInstance]),
    fetchTicker: vi.fn().mockResolvedValue(mockTickerFactory('BTC/USDT')),
    fetchTickers: vi.fn().mockResolvedValue({ 'BTC/USDT': mockTickerFactory('BTC/USDT') }),
    fetchOHLCV: vi.fn().mockResolvedValue([[Date.now(), 1, 2, 0, 1, 100]] as OHLCV[]),
    fetchOrderBook: vi.fn().mockResolvedValue({ symbol: 'BTC/USDT', bids: [], asks: [], timestamp: MOCK_TIMESTAMP, datetime: new Date(MOCK_TIMESTAMP).toISOString(), nonce: undefined }),
    createOrder: vi.fn().mockResolvedValue(mockOrderFactory()), 
    cancelOrder: vi.fn().mockResolvedValue(undefined), 
    fetchOrder: vi.fn().mockResolvedValue(mockOrderFactory()),
    fetchOpenOrders: vi.fn().mockResolvedValue([mockOrderFactory()]),
    fetchClosedOrders: vi.fn().mockResolvedValue([mockOrderFactory({ status: 'closed' })]),
    fetchMyTrades: vi.fn().mockResolvedValue([mockTradeFactory()]),
    fetchTrades: vi.fn().mockResolvedValue([mockTradeFactory()]), 
    fetchBalance: vi.fn().mockResolvedValue(mockBalance),
    fetchPositions: vi.fn().mockResolvedValue(mockPositions),
    fetchFundingRate: vi.fn().mockResolvedValue(MOCK_FUNDING_RATE_INFO_FACTORY()),
    fetchFundingRates: vi.fn().mockResolvedValue({ [symbolForFundingRatesKey]: MOCK_FUNDING_RATE_INFO_FACTORY() }),
    fetchFundingRateHistory: vi.fn().mockResolvedValue([MOCK_CCXT_FUNDING_RATE_HISTORY_ITEM_FACTORY()]),
    setLeverage: vi.fn().mockResolvedValue({ info: 'Leverage set' }), // Simplified return for setLeverage
    transfer: vi.fn().mockResolvedValue(MOCK_CCXT_TRANSFER_ENTRY_FACTORY()),
    fetchTradingFees: vi.fn().mockResolvedValue(MOCK_TRADING_FEES_FACTORY()),
    has: {
      fetchMarkets: true, fetchOHLCV: true, fetchTicker: true, fetchTickers: true,
      fetchOrderBook: true, createOrder: true, cancelOrder: true, fetchBalance: true,
      fetchMyTrades: true, fetchPositions: true, fetchFundingRate: true, setLeverage: true,
      fetchTrades: true, fetchOrder: true, fetchOpenOrders: true, fetchClosedOrders: true,
      fetchFundingRates: true, fetchFundingRateHistory: true, transfer: true, fetchTradingFees: true,
    },
    markets: { [pair]: marketDefaultsForPair } as Record<TradingPairSymbol, Market>,
    currencies: { 
      [base]: { id: base, code: base, precision: 8, name: base } as CCXTCurrency,
      [quote]: { id: quote, code: quote, precision: 8, name: quote } as CCXTCurrency,
    },
    options: {},
    verbose: false,
  } as MockExchangeInstance; 
}

export const ALL_MOCK_EXCHANGE_IDS: ExchangeId[] = [
  'binance', 'binanceusdm', 'bybit', 'bitget', 'bingx', 'coinbase', 'gateio', 'kraken', 'kucoin', 'mexc', 'okx', 'phemex',
];

// Store exchanges instances that will be accessible for tests to modify
export const _testMockInstances: Partial<Record<ExchangeId, MockExchangeInstance>> = {};

export function createMockExchangeService(): { 
  exchangeService: ExchangeService;
  mockLogger: Logger;
  singleMockExchangeInstance: MockExchangeInstance;
} {
  // Create mock instance for tests
  const singleMockExchangeInstance = createMockInstance('binance' as ExchangeId);
  
  // Store in our local accessible mock instances
  _testMockInstances['binance' as ExchangeId] = singleMockExchangeInstance;
  
  // Create mock env and logger
  const mockLogger = {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    fatal: vi.fn()
  } as unknown as Logger;
  
  const mockKV = {
    get: vi.fn().mockImplementation(async () => null),
    put: vi.fn().mockResolvedValue(undefined),
    delete: vi.fn().mockResolvedValue(undefined),
    list: vi.fn().mockResolvedValue({ keys: [] }),
  };
  
  const mockEnv = {
    ArbEdgeKV: mockKV,
  };
  
  // Get ExchangeService constructor
  const { ExchangeService } = require('../../src/services/exchangeService');
  
  // Create an instance with our mocks
  const exchangeService = new ExchangeService({
    env: mockEnv,
    logger: mockLogger,
  });
  
  // Make instance directly available for tests
  exchangeService.exchangeInstances = new Map();
  exchangeService.exchangeInstances.set('binance', singleMockExchangeInstance);
  
  return { exchangeService, mockLogger, singleMockExchangeInstance };
}

