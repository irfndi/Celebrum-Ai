// Re-export or define types here

export type ExchangeId = 'binance' | 'bybit' | 'kraken' | 'mexc'; // Add other exchanges as needed
export type TradingPairSymbol = string; // e.g., 'BTC/USDT'

// Define the structured pair type
export interface StructuredTradingPair {
  base: string;
  quote: string;
  symbol: TradingPairSymbol; // Keep the original string format as well
}

export interface FundingRateInfo {
  symbol: StructuredTradingPair; // Use the structured type
  fundingRate: number;
  timestamp: number;
  datetime: string;
  info: Record<string, unknown>; // Raw exchange response - consider typing more strictly if possible
}

export interface ArbitrageOpportunity {
  pair: TradingPairSymbol;
  longExchange: ExchangeId;
  shortExchange: ExchangeId;
  longRate: number;
  shortRate: number;
  rateDifference: number;
  timestamp: number;
}

// Add other types as needed (e.g., Balance, Position, Order)
export interface Balance {
    currency: string;
    total: number;
    free: number;
    used: number;
}

export interface Position {
    symbol: TradingPairSymbol;
    side: 'long' | 'short';
    entryPrice: number;
    markPrice: number;
    amount?: number; // Size of the position
    contracts?: number;
    margin: number;
    pnl: number;
    leverage: number;
    info: Record<string, unknown>; // Raw exchange response
}

export interface Order {
    id: string;
    symbol: TradingPairSymbol;
    type: 'limit' | 'market';
    side: 'buy' | 'sell';
    amount: number;
    price?: number; // Undefined for market orders
    status: 'open' | 'closed' | 'canceled' | 'expired';
    timestamp: number;
    info: Record<string, unknown>; // Raw exchange response
}