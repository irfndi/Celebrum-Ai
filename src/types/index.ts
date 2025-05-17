// Re-export or define types here

export type ExchangeId =
  | 'binance'
  | 'bybit'
  | 'kraken'
  | 'mexc'
  | 'okx'
  | 'bingx'
  | 'bitget'
  | (string & {}); // allow custom IDs while keeping autocomplete for known ones // Add other exchanges as needed
export type TradingPairSymbol = string; // e.g., 'BTC/USDT'

export type OrderType = 'limit' | 'market'; // Added OrderType
export type PositionSide = 'long' | 'short'; // Added PositionSide
export type OrderSide = 'buy' | 'sell'; // Added OrderSide

// Basic Env interface for Cloudflare Workers
export interface Env {
  KV_NAMESPACE: KVNamespace;
  // Add other bindings as needed (e.g., D1 databases, R2 buckets, secrets)
}

// Forward declaration for KVNamespace to be used in Env before its full import elsewhere if needed
// This helps avoid direct import of '@cloudflare/workers-types' in this central types file
// if it's only needed for the Env interface structure.
// Consumers will import the actual KVNamespace type from '@cloudflare/workers-types'.
interface KVNamespace { readonly _: unique symbol; }


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
  rateDifference: number; // Gross difference before fees
  longExchangeTakerFeeRate: number;
  shortExchangeTakerFeeRate: number;
  totalEstimatedFees: number; // Combined percentage cost (e.g., 0.001 for 0.1% + 0.1% = 0.002)
  netRateDifference: number; // rateDifference - totalEstimatedFees
  timestamp: number;
}

// Add other types as needed (e.g., Balance, Position, Order)
export type Balances = Record<string, Balance>;

export interface Balance {
    currency: string;
    total: number;
    free: number;
    used: number;
    info?: Record<string, unknown>; // Optional: Raw exchange response or additional details
}

export interface Position {
    symbol: TradingPairSymbol;
    side: PositionSide;
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
    type: OrderType;
    side: 'buy' | 'sell';
    amount: number;
    price?: number; // Undefined for market orders
    status: 'open' | 'closed' | 'canceled' | 'expired';
    timestamp: number;
    info: Record<string, unknown>; // Raw exchange response
}

// Basic Market interface (can be expanded based on ccxt structure)
export interface Market {
  id: string;
  symbol: TradingPairSymbol;
  base: string;
  quote: string;
  baseId: string;
  quoteId: string;
  active: boolean | undefined;
  type: string;
  spot: boolean;
  margin: boolean;
  swap: boolean;
  future: boolean;
  option: boolean;
  contract: boolean;
  settle: string | undefined;
  settleId: string | undefined;
  contractSize: number | undefined;
  listingTime: number | undefined;
  expiry: number | undefined;
  expiryDatetime: string | undefined;
  strike: number | undefined;
  optionType: string | undefined;
  precision: {
    amount: number | undefined;
    price: number | undefined;
    base: number | undefined;
    quote: number | undefined;
  };
  limits: {
    leverage: {
      min: number | undefined;
      max: number | undefined;
    };
    amount: {
      min: number | undefined;
      max: number | undefined;
    };
    price: {
      min: number | undefined;
      max: number | undefined;
    };
    cost: {
      min: number | undefined;
      max: number | undefined;
    };
  };
  info: Record<string, unknown>; // Raw exchange response
}

// Basic Ticker interface (can be expanded based on ccxt structure)
export interface Ticker {
  symbol: TradingPairSymbol;
  timestamp: number | undefined;
  datetime: string | undefined;
  high: number | undefined;
  low: number | undefined;
  bid: number | undefined;
  bidVolume: number | undefined;
  ask: number | undefined;
  askVolume: number | undefined;
  vwap: number | undefined;
  open: number | undefined;
  close: number | undefined;
  last: number | undefined;
  previousClose: number | undefined;
  change: number | undefined;
  percentage: number | undefined;
  average: number | undefined;
  baseVolume: number | undefined;
  quoteVolume: number | undefined;
  info: Record<string, unknown>; // Raw exchange response
}