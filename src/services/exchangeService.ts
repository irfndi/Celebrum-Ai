import * as ccxt from 'ccxt';
import type { ExchangeId, TradingPairSymbol, FundingRateInfo, Balance, Position, Order } from '../types';

export class ExchangeService {
  private exchanges: Map<string, ccxt.Exchange> = new Map();

  private getExchangeInstance(exchangeId: string): ccxt.Exchange {
    if (!this.exchanges.has(exchangeId)) {
      // Ensure the exchange ID is valid according to ccxt
      if (!(exchangeId in ccxt)) {
        throw new Error(`Unsupported exchange: ${exchangeId}`);
      }
      // TODO: Add API key/secret handling from secrets
      const exchangeOptions = {
        // apiKey: 'YOUR_API_KEY', // Load from secrets
        // secret: 'YOUR_API_SECRET', // Load from secrets
      };
      // Use ccxt.pro for WebSocket support if needed later
      // Type assertion for the constructor
      // Type assertion via unknown
      // Type assertion via unknown, using unknown for options
      const ExchangeConstructor = (ccxt as unknown as Record<string, new (options?: Record<string, unknown>) => ccxt.Exchange>)[exchangeId];
      if (!ExchangeConstructor) {
          throw new Error(`Invalid exchange ID or constructor not found in ccxt library: ${exchangeId}`);
      }
      const exchange = new ExchangeConstructor(exchangeOptions);
      this.exchanges.set(exchangeId, exchange);
      // It's often necessary to load markets before making other calls
      // Consider doing this lazily or upfront
      // await exchange.loadMarkets();
    }
    const exchange = this.exchanges.get(exchangeId);
    if (!exchange) {
      // This should theoretically not happen based on the logic above,
      // but it satisfies the linter and adds robustness.
      throw new Error(`Failed to retrieve exchange instance for ${exchangeId} after creation.`);
    }
    return exchange;
  }

  async getFundingRate(exchangeId: string, symbol: string): Promise<FundingRateInfo | null> {
    try {
      const exchange = this.getExchangeInstance(exchangeId);
      // Ensure markets are loaded before fetching funding rate
      // This might be redundant if loaded in getExchangeInstance,
      // but ccxt handles internal caching.
      await exchange.loadMarkets(); // Ensure markets are loaded

      console.log(`Fetching funding rate for ${symbol} on ${exchangeId}`);
      const rate = await exchange.fetchFundingRate(symbol);

      // Basic validation: check if essential data (including datetime string) is present
      if (
        rate &&
        typeof rate.fundingRate === 'number' &&
        typeof rate.timestamp === 'number' &&
        typeof rate.datetime === 'string' && // Ensure datetime is a string
        rate.datetime.length > 0 // Optional: check if not empty
      ) {
        // Parse the symbol and return the necessary fields matching FundingRateInfo
        const [base, quote] = symbol.split('/'); // Assuming standard 'BASE/QUOTE' format
        return { 
          symbol: { base, quote, symbol: symbol as TradingPairSymbol }, // Construct the StructuredTradingPair object
          fundingRate: rate.fundingRate,
          timestamp: rate.timestamp,
          datetime: rate.datetime, 
          info: rate.info, // Include info if expected by tests/consumers
        };
      }

      // If the validation above failed, log error and return null
      console.error(`Invalid funding rate data received for ${symbol} on ${exchangeId}:`, rate);
      return null;

    } catch (error) {
      console.error(`Error fetching funding rate for ${symbol} on ${exchangeId}:`, error);
      return null;
    }
  }

  async getBalance(exchangeId: string, currency: string): Promise<number | null> {
    try {
      const exchange = this.getExchangeInstance(exchangeId);
      await exchange.loadMarkets(); // Ensure markets are loaded

      console.log(`Fetching balance for ${currency} on ${exchangeId}`);
      const balance = await exchange.fetchBalance();

      // Cast balance.total to a more flexible type for indexing, converting to unknown first as suggested
      const totalBalance = balance?.total as unknown as Record<string, number> | undefined;

      if (!balance || !totalBalance || typeof totalBalance[currency] === 'undefined') {
        console.error(`Could not retrieve total balance for ${currency} on ${exchangeId}:`, balance);
        return null;
      }

      // Return the total balance for the specified currency
      return totalBalance[currency];

    } catch (error) {
      console.error(`Error fetching balance for ${currency} on ${exchangeId}:`, error);
      return null;
    }
  }

  async getOpenPositions(exchangeId: string, symbol?: TradingPairSymbol): Promise<ccxt.Position[] | null> {
    try {
      const exchange = this.getExchangeInstance(exchangeId);
      await exchange.loadMarkets(); // Ensure markets are loaded

      console.log(`Fetching open positions on ${exchangeId}${symbol ? ` for ${symbol}` : ''}`);
      // Pass symbol as an array if defined, otherwise undefined for all symbols
      const symbolsToFetch = symbol ? [symbol] : undefined;
      const positions: ccxt.Position[] = await exchange.fetchPositions(symbolsToFetch);

      // Basic validation
      if (!Array.isArray(positions)) {
        console.error(`Invalid positions data received from ${exchangeId}:`, positions);
        return null;
      }

      // Filter for open positions if needed, although fetchPositions usually returns only open ones
      // Depending on the exchange and ccxt implementation, you might need to filter
      // const openPositions = positions.filter(pos => pos.isOpen);

      return positions;

    } catch (error) {
      console.error(`Error fetching open positions on ${exchangeId}${symbol ? ` for ${symbol}` : ''}:`, error);
      return null;
    }
  }

  async placeOrder(exchangeId: string, symbol: TradingPairSymbol, type: ccxt.OrderType, side: ccxt.OrderSide, amount: number, price?: number): Promise<ccxt.Order | null> {
    try {
      const exchange = this.getExchangeInstance(exchangeId);
      await exchange.loadMarkets(); // Ensure markets are loaded

      console.log(`Placing ${side} ${type} order for ${amount} of ${symbol} on ${exchangeId}${price ? ` at price ${price}` : ''}`);

      // ccxt's createOrder method
      const order = await exchange.createOrder(symbol, type, side, amount, price);

      // Basic validation
      if (!order || !order.id) {
        console.error(`Invalid order data received from ${exchangeId}:`, order);
        return null;
      }

      return order;

    } catch (error) {
      console.error(`Error placing order on ${exchangeId} for ${symbol}:`, error);
      return null;
    }
  }

  /**
   * Closes an existing open position on a specified exchange.
   * @param exchangeId The ID of the exchange (e.g., 'binance', 'bybit').
   * @param position The Position object to close.
   * @returns {Promise<ccxt.Order | null>} The created order object for closing the position, or null if an error occurs.
   */
  async closePosition(
    exchangeId: string,
    position: Position
  ): Promise<ccxt.Order | null> {
    try {
      const exchange = this.getExchangeInstance(exchangeId);
      await exchange.loadMarkets(); // Ensure markets are loaded

      console.log(`Closing position for ${position.symbol} on ${exchangeId}`);

      // Determine the opposite side for closing
      const side = position.side === 'long' ? 'sell' : 'buy';
      const type = 'market'; // Usually close positions with market orders for speed

      // ccxt's createOrder method to close a position
      // Note: Some exchanges might have specific methods for closing positions.
      // createOrder with the opposite side and the position size is a common way.
      // Assuming position contracts/quantity is available in the position object
      // Use optional chaining and nullish coalescing with the updated Position type
      const positionSize = position.contracts ?? position.amount ?? 0;
      const order = await exchange.createOrder(position.symbol, type, side, positionSize);

      // Basic validation
      if (!order || !order.id) {
        console.error(`Invalid order data received from ${exchangeId}:`, order);
        return null;
      }

      return order;

    } catch (error) {
      console.error(`Error closing position on ${exchangeId} for ${position.symbol}:`, error);
      return null;
    }
  }

  // TODO: Add methods for other functionalities (e.g., fetchOrder, cancelOrder)
}
