import type { TradingPairSymbol } from '../../src/types';
import { MOCK_TIMESTAMP } from './mockUtils';

// This is a simplified Ticker structure. Adapt as needed based on ccxt.Ticker or your local Ticker type.
export interface MockTicker {
  symbol: TradingPairSymbol;
  timestamp: number;
  datetime: string;
  high?: number;
  low?: number;
  bid?: number;
  bidVolume?: number;
  ask?: number;
  askVolume?: number;
  vwap?: number;
  open?: number;
  close?: number;
  last?: number;
  previousClose?: number;
  change?: number;
  percentage?: number;
  average?: number;
  baseVolume?: number;
  quoteVolume?: number;
  info: Record<string, unknown>;
}

export const MOCK_TICKER_DEFAULTS: MockTicker = {
  symbol: 'BTC/USDT' as TradingPairSymbol,
  timestamp: MOCK_TIMESTAMP,
  datetime: new Date(MOCK_TIMESTAMP).toISOString(),
  last: 30000,
  info: { mockTickerInfo: 'default_ticker_details' },
};

export const mockTickerFactory = (symbol?: TradingPairSymbol, overrides: Partial<MockTicker> = {}): MockTicker => {
  return {
    ...MOCK_TICKER_DEFAULTS,
    symbol: symbol || MOCK_TICKER_DEFAULTS.symbol,
    timestamp: Date.now(), // Use current time for dynamic tickers
    datetime: new Date(Date.now()).toISOString(),
    ...overrides,
    info: { ...MOCK_TICKER_DEFAULTS.info, ...(overrides.info || {}) },
  };
};
