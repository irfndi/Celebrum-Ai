import type { Order, TradingPairSymbol, OrderSide, OrderType } from '../../src/types';
import { MOCK_TIMESTAMP } from './mockUtils';

export const MOCK_ORDER_DEFAULTS: Order = {
  id: `order_${MOCK_TIMESTAMP}`,
  symbol: 'BTC/USDT' as TradingPairSymbol,
  type: 'limit' as OrderType,
  side: 'buy' as OrderSide,
  amount: 1,
  price: 29000, // Optional, but good to have a default for limit orders
  status: 'open',
  timestamp: MOCK_TIMESTAMP,
  // datetime: new Date(MOCK_TIMESTAMP).toISOString(), // Not in local Order type as per Step 62 view
  info: { detail: 'mock_order_default_info' },
  // Fields not in local Order: clientOrderId, lastTradeTimestamp, timeInForce, average, filled, remaining, cost, fee, trades, reduceOnly, postOnly
};

export const mockOrderFactory = (options: Partial<Order> = {}): Order => {
  const defaults: Order = {
    ...MOCK_ORDER_DEFAULTS,
    id: `order_${Date.now()}`,
    clientOrderId: `client_order_${Date.now()}`,
    timestamp: Date.now(),
    datetime: new Date(Date.now()).toISOString(),
    info: options.info ? { ...options.info } : { ...MOCK_ORDER_DEFAULTS.info },
  };
  return { ...defaults, ...options, info: options.info ? { ...defaults.info, ...options.info } : defaults.info };
};

// Mock OrderBook
export const mockOrderBook = {
  bids: [[29999, 1.5], [29998, 2.5]],
  asks: [[30001, 0.5], [30002, 1.2]],
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  nonce: undefined,
  symbol: 'BTC/USDT' as TradingPairSymbol,
};
