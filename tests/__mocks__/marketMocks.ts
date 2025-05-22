import type { Market } from '../../src/types';
import type { MarketInterface as CCXTMarketInterface } from 'ccxt';

// Assuming Market type is effectively ccxt.Market or similar
// For MOCK_MARKET_DEFAULTS, we use Partial<Market> to allow only a subset of fields
// and add 'created' as an optional number because it's set dynamically.
export const MOCK_MARKET_DEFAULTS: Partial<Market> & { created?: number; marginMode?: string; marginType?: string; } = {
  margin: false,
  id: 'BTC/USDT',
  symbol: 'BTC/USDT',
  base: 'BTC',
  quote: 'USDT',
  baseId: 'BTC',
  quoteId: 'USDT',
  active: true,
  type: 'spot',
  linear: undefined,
  inverse: undefined,
  spot: true,
  swap: false,
  future: false,
  option: false,
  contract: false, // This market is a spot market, so it's not a contract itself.
  // 'contracts' field removed based on lint error e75021da-6858-47b6-bea1-7a1d2124bc26.
  // If the Market type truly supports 'contracts', this mock might need it for non-spot types.
  expiry: undefined,
  expiryDatetime: undefined,
  strike: undefined,
  optionType: undefined,
  taker: 0.001,
  maker: 0.001,
  percentage: false, // Assuming percentage is boolean (e.g. for fees)
  marginMode: 'isolated', // Kept as in original mock, assuming it's used (perhaps custom field)
  marginType: 'linear',   // Kept as in original mock, assuming it's used (perhaps custom field)
  feeSide: 'quote',
  precision: { amount: 8, price: 2 },
  limits: {
    leverage: { min: 1, max: 100 },
    amount: { min: 0.00001, max: 1000 },
    price: { min: 0.01, max: 1000000 },
    cost: { min: 1, max: 10000000 },
  },
  // 'created' will be set by createMockMarket if not overridden.
  info: {},
};

export const createMockMarket = (overrides?: Partial<Market>): Market => {
  // Start with a deep copy of defaults, explicitly typed
  const market: Market = JSON.parse(JSON.stringify(MOCK_MARKET_DEFAULTS)) as Market;

  if (overrides) {
    for (const key in overrides) {
      if (Object.prototype.hasOwnProperty.call(overrides, key)) {
        const k = key as keyof Market;
        const overrideValue = overrides[k];

        if ((k === 'precision' || k === 'limits')) {
          const currentMarketSubObject = market[k]; // Store in a variable for type guarding
          if (currentMarketSubObject != null && typeof currentMarketSubObject === 'object') {
            // Ensure market[k] is treated as an object for merging
            // And overrideValue is an object or null/undefined (in which case, spread {} for overrideValue)
            market[k] = {
              ...currentMarketSubObject, // Use the guarded variable
              ...(overrideValue != null && typeof overrideValue === 'object' ? overrideValue : {}),
            } as Market[typeof k];
          }
        } else if (overrideValue !== undefined) {
          // For other properties, directly assign if the overrideValue is not undefined.
          // This allows overriding with null, false, 0 etc.
          market[k] = overrideValue as Market[typeof k];
        }
      }
    }
  }

  // Handle 'created' timestamp
  // If market.created is still undefined (i.e., not set by overrides), set it to now.
  if (market.created === undefined) {
    market.created = Date.now();
  }
  return market;
};
