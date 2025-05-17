// Assuming a local 'Trade' type will be similar to ccxt.Trade or defined later in src/types
// For now, define a local simplified Trade type for the mock.
import type { TradingPairSymbol, OrderSide, OrderType } from '../../src/types';
import { MOCK_TIMESTAMP } from './mockUtils';

// Placeholder for local Trade type - adjust when defined in src/types/index.ts
export interface MockLocalTrade {
  id: string;
  order?: string;
  timestamp: number;
  datetime: string;
  symbol: TradingPairSymbol;
  type?: OrderType;
  side: OrderSide;
  takerOrMaker?: 'taker' | 'maker';
  price: number;
  amount: number;
  cost: number;
  fee?: { currency: string; cost: number; rate: number };
  info: Record<string, unknown>; // Changed 'any' to 'unknown'
}

export const MOCK_TRADE_DEFAULTS: MockLocalTrade = { // Use MockLocalTrade
  id: `trade_${MOCK_TIMESTAMP}`,
  order: `order_${MOCK_TIMESTAMP}`,
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  symbol: 'BTC/USDT' as TradingPairSymbol,
  type: 'limit' as OrderType,
  side: 'buy' as OrderSide,
  takerOrMaker: 'taker',
  price: 29500,
  amount: 0.5,
  cost: 29500 * 0.5,
  fee: { currency: 'USDT', cost: (29500 * 0.5) * 0.001, rate: 0.001 },
  info: { detail: 'mock_trade_default_info' },
};

export const mockTradeFactory = (options: Partial<MockLocalTrade> = {}): MockLocalTrade => { // Use MockLocalTrade
  const defaults: MockLocalTrade = { // Use MockLocalTrade
    ...MOCK_TRADE_DEFAULTS,
    id: `trade_${Date.now()}`,
    timestamp: Date.now(),
    datetime: new Date(Date.now()).toISOString(),
    info: options.info ? { ...options.info } : { ...MOCK_TRADE_DEFAULTS.info },
  };
  return { ...defaults, ...options, info: options.info ? { ...defaults.info, ...options.info } : defaults.info };
};
