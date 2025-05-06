import { describe, it, expect, beforeEach, vi } from 'vitest'; // Add vi import
import type { Mock } from 'vitest';
import { ExchangeService } from '../../src/services/exchangeService';
import * as ccxt from 'ccxt'; // Revert to regular import
import type { Position } from '../../src/types'; // Import local Position type

// Define types for the mocked structure more loosely
type MockExchangeInstance = {
  fetchFundingRate: Mock<(symbol: string) => Promise<Record<string, unknown>>>; // Use Record<string, unknown> instead of any
  loadMarkets: Mock<() => Promise<Record<string, unknown>>>; // Use Record<string, unknown> instead of any
  fetchBalance: Mock<() => Promise<Record<string, unknown>>>; // Add mock for fetchBalance
  fetchPositions: Mock<(symbol?: string) => Promise<Record<string, unknown>[]>>; // Add mock for fetchPositions
  createOrder: Mock<(symbol: string, type: ccxt.OrderType, side: ccxt.OrderSide, amount: number, price?: number) => Promise<Record<string, unknown>>>; // Add mock for createOrder
  // Add other mocked methods if needed
};

// Mock ccxt library - Further Refined structure
vi.mock('ccxt', () => {
  const mockInstance: MockExchangeInstance = {
    fetchFundingRate: vi.fn(),
    loadMarkets: vi.fn().mockResolvedValue({}), // Mock market loading
    fetchBalance: vi.fn(), // Initialize mock for fetchBalance
    fetchPositions: vi.fn(), // Initialize mock for fetchPositions
    createOrder: vi.fn(), // Initialize mock for createOrder
  };
  const mockConstructor = vi.fn().mockImplementation(() => mockInstance);

  return {
    // Add top-level exchange constructors for the 'in' check
    binance: mockConstructor,
    bybit: mockConstructor,

    // Keep the default export structure as well if it's used elsewhere
    // (though the service currently uses the top-level check)
    default: {
      binance: mockConstructor,
      bybit: mockConstructor,
      version: 'mock-version',
      exchanges: ['binance', 'bybit'],
      // ... other necessary properties
    },
    // Add other top-level exports if needed by the service (e.g., Exchange class)
    // Exchange: class MockExchange {},
  };
});

/* // TODO: Fix or remove this unreachable test
  it('should place an order on Binance', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<[], MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor(); // Get the instance

    // Add createOrder to the mock instance
    mockBinanceInstance.createOrder = vi.fn();

    const expectedOrder = {
      id: '12345',
      symbol: 'BTC/USDT',
      type: 'limit',
      side: 'buy',
      amount: 0.01,
      price: 50000,
      status: 'open',
      info: {}
    };
    mockBinanceInstance.createOrder.mockResolvedValue(expectedOrder);

    const service = new ExchangeService();
    const order = await service.placeOrder('binance', 'BTC/USDT', 'limit', 'buy', 0.01, 50000);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('BTC/USDT', 'limit', 'buy', 0.01, 50000);
    expect(order).toEqual(expectedOrder);
  });

  it('should close a position on Binance', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<[], MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    // Add createOrder to the mock instance if not already added
    if (!mockBinanceInstance.createOrder) {
      mockBinanceInstance.createOrder = vi.fn();
    }

    const position = {
      symbol: 'BTC/USDT',
      side: 'long',
      contracts: 0.01,
      entryPrice: 60000,
      unrealizedPnl: 100,
      margin: 500,
      info: {}
    };

    const expectedOrder = {
      id: '67890',
      symbol: 'BTC/USDT',
      type: 'market',
      side: 'sell',
      amount: 0.01,
      status: 'closed',
      info: {}
    };
    mockBinanceInstance.createOrder.mockResolvedValue(expectedOrder);

    const service = new ExchangeService();
    const order = await service.closePosition('binance', position as ccxt.Position);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('BTC/USDT', 'market', 'sell', 0.01);
    expect(order).toEqual(expectedOrder);
  });
*/


describe('ExchangeService', () => {
  it('should be defined (placeholder test)', () => {
    // This is just a placeholder to ensure the file is created
    expect(true).toBe(true);
  });

  it('should fetch funding rate for a given pair from Binance', async () => {
    // Access the mocked constructor/instance (now directly available)
    // We still need the cast because the mock structure is simplified
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor(); // Get the instance

    // Data the mock function will return
    const mockRateData = {
      symbol: 'BTC/USDT',
      fundingRate: 0.0001,
      timestamp: 1700000000000, // Use a concrete number for the mock
      datetime: new Date(1700000000000).toISOString(), // Add datetime to mock return
      info: {}, 
      // Add other relevant fields returned by ccxt if the mock provides them
    };

    // Expected shape for the assertion
    const expectedRateResult = {
      symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, // Match TradingPair structure used in service
      fundingRate: 0.0001,
      timestamp: 1700000000000,
      datetime: new Date(1700000000000).toISOString(), // Add datetime
      info: {}
    };
    // Ensure the mock function on the instance is set up
    mockBinanceInstance.fetchFundingRate.mockResolvedValue(mockRateData);

    const service = new ExchangeService();
    const rate = await service.getFundingRate('binance', 'BTC/USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled(); // Ensure markets are loaded
    expect(mockBinanceInstance.fetchFundingRate).toHaveBeenCalledWith('BTC/USDT');
    expect(rate).toEqual(expectedRateResult); // Assert against the expected shape
  });

  it('should fetch balance for a given currency from Binance', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const expectedBalance = {
      total: {
        USDT: 1000.50,
        BTC: 0.1,
      },
      // Include other balance properties if necessary for the test
    };
    mockBinanceInstance.fetchBalance.mockResolvedValue(expectedBalance);

    const service = new ExchangeService();
    const balance = await service.getBalance('binance', 'USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchBalance).toHaveBeenCalled();
    expect(balance).toBe(expectedBalance.total.USDT);
  });

  it('should fetch balance for a given currency from Bybit', async () => {
    const MockedBybitConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).bybit;
    const mockBybitInstance = MockedBybitConstructor();

    const expectedBalance = {
      total: {
        USDT: 500.75,
        ETH: 0.5,
      },
    };
    mockBybitInstance.fetchBalance.mockResolvedValue(expectedBalance);

    const service = new ExchangeService();
    const balance = await service.getBalance('bybit', 'ETH');

    expect(mockBybitInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBybitInstance.fetchBalance).toHaveBeenCalled();
    expect(balance).toBe(expectedBalance.total.ETH);
  });

  it('should return null and log error if fetching funding rate fails', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const errorMessage = 'Failed to fetch funding rate';
    mockBinanceInstance.fetchFundingRate.mockRejectedValue(new Error(errorMessage));

    // Spy on console.error
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const service = new ExchangeService();
    const rate = await service.getFundingRate('binance', 'BTC/USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchFundingRate).toHaveBeenCalledWith('BTC/USDT');
    expect(rate).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error fetching funding rate for BTC/USDT on binance:',
      expect.any(Error)
    );

    consoleErrorSpy.mockRestore(); // Restore console.error
  });

  it('should return null and log error if fetching balance fails', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const errorMessage = 'Failed to fetch balance';
    mockBinanceInstance.fetchBalance.mockRejectedValue(new Error(errorMessage));

    // Spy on console.error
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const service = new ExchangeService();
    const balance = await service.getBalance('binance', 'USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchBalance).toHaveBeenCalled();
    expect(balance).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error fetching balance for USDT on binance:',
      expect.any(Error)
    );

    consoleErrorSpy.mockRestore(); // Restore console.error
  });

  it('should fetch open positions for a given pair from Binance', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const expectedPositions = [
      { symbol: 'BTC/USDT', side: 'long', size: 0.01, entryPrice: 60000, unrealizedPnl: 100, margin: 500, info: {} },
      // Add other position properties if necessary
    ];
    mockBinanceInstance.fetchPositions.mockResolvedValue(expectedPositions);

    const service = new ExchangeService();
    const positions = await service.getOpenPositions('binance', 'BTC/USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchPositions).toHaveBeenCalledWith(['BTC/USDT']);
    expect(positions).toEqual(expectedPositions);
  });

  it('should fetch all open positions from Binance if no symbol is provided', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const expectedPositions = [
      { symbol: 'BTC/USDT', side: 'long', size: 0.01, entryPrice: 60000, unrealizedPnl: 100, margin: 500, info: {} },
      { symbol: 'ETH/USDT', side: 'short', size: 0.1, entryPrice: 3000, unrealizedPnl: -50, margin: 300, info: {} },
    ];
    mockBinanceInstance.fetchPositions.mockResolvedValue(expectedPositions);

    const service = new ExchangeService();
    const positions = await service.getOpenPositions('binance');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchPositions).toHaveBeenCalledWith(undefined); // Should be called with no argument or undefined
    expect(positions).toEqual(expectedPositions);
  });

  it('should return null and log error if fetching open positions fails', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    const errorMessage = 'Failed to fetch positions';
    mockBinanceInstance.fetchPositions.mockRejectedValue(new Error(errorMessage));

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const service = new ExchangeService();
    const positions = await service.getOpenPositions('binance', 'BTC/USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchPositions).toHaveBeenCalledWith(['BTC/USDT']);
    expect(positions).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error fetching open positions on binance for BTC/USDT:',
      expect.any(Error)
    );

    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if fetching open positions returns invalid data', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();

    // Mock fetchPositions to return non-array data
    const invalidData = { message: 'Invalid data format' };
    // biome-ignore lint/suspicious/noExplicitAny: Testing invalid fetchPositions response
    mockBinanceInstance.fetchPositions.mockResolvedValue(invalidData as any); // Revert cast back to any for this mock

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const service = new ExchangeService();
    const positions = await service.getOpenPositions('binance', 'BTC/USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchPositions).toHaveBeenCalledWith(['BTC/USDT']);
    expect(positions).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Invalid positions data received from binance:',
      invalidData
    );

    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if fetchBalance returns null', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    // biome-ignore lint/suspicious/noExplicitAny: Testing null fetchBalance response
    mockBinanceInstance.fetchBalance.mockResolvedValue(null as any); // Return null

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const balance = await service.getBalance('binance', 'USDT');

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.fetchBalance).toHaveBeenCalled();
    expect(balance).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Could not retrieve total balance for USDT on binance:',
      null
    );
    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if fetchBalance response lacks total property', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const incompleteBalance = { free: { USDT: 100 } }; // Missing 'total'
    // biome-ignore lint/suspicious/noExplicitAny: Testing incomplete fetchBalance response
    mockBinanceInstance.fetchBalance.mockResolvedValue(incompleteBalance as any);

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const balance = await service.getBalance('binance', 'USDT');

    expect(balance).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Could not retrieve total balance for USDT on binance:',
      incompleteBalance
    );
    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if fetchBalance total lacks requested currency', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const wrongCurrencyBalance = {
      total: { BTC: 0.1 }, // Missing 'USDT'
    };
    // biome-ignore lint/suspicious/noExplicitAny: Testing wrong currency fetchBalance response
    mockBinanceInstance.fetchBalance.mockResolvedValue(wrongCurrencyBalance as any);

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const balance = await service.getBalance('binance', 'USDT');

    expect(balance).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Could not retrieve total balance for USDT on binance:',
      wrongCurrencyBalance
    );
    consoleErrorSpy.mockRestore();
  });

  // --- placeOrder Tests ---

  it('should place a limit order successfully', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const expectedOrder = {
      id: 'order123',
      symbol: 'BTC/USDT',
      type: 'limit',
      side: 'buy',
      amount: 0.01,
      price: 60000,
      info: { status: 'open' },
    };
    // biome-ignore lint/suspicious/noExplicitAny: Testing successful placeOrder response format
    mockBinanceInstance.createOrder.mockResolvedValue(expectedOrder as any);

    const service = new ExchangeService();
    const order = await service.placeOrder('binance', 'BTC/USDT', 'limit', 'buy', 0.01, 60000);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('BTC/USDT', 'limit', 'buy', 0.01, 60000);
    expect(order).toEqual(expectedOrder);
  });

  it('should place a market order successfully', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const expectedOrder = {
      id: 'order456',
      symbol: 'ETH/USDT',
      type: 'market',
      side: 'sell',
      amount: 0.1,
      info: { status: 'filled' },
    };
    // biome-ignore lint/suspicious/noExplicitAny: Testing successful market placeOrder response format
    mockBinanceInstance.createOrder.mockResolvedValue(expectedOrder as any);

    const service = new ExchangeService();
    // Price is undefined for market order
    const order = await service.placeOrder('binance', 'ETH/USDT', 'market', 'sell', 0.1);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('ETH/USDT', 'market', 'sell', 0.1, undefined);
    expect(order).toEqual(expectedOrder);
  });

  it('should return null and log error if placeOrder returns invalid data (no id)', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const invalidOrderData = { symbol: 'BTC/USDT', amount: 0.01 }; // Missing id
    // biome-ignore lint/suspicious/noExplicitAny: Testing invalid placeOrder response (missing id)
    mockBinanceInstance.createOrder.mockResolvedValue(invalidOrderData as any);

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const order = await service.placeOrder('binance', 'BTC/USDT', 'market', 'buy', 0.01);

    expect(order).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Invalid order data received from binance:',
      invalidOrderData
    );
    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if placeOrder throws an error', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const errorMessage = 'Insufficient balance';
    mockBinanceInstance.createOrder.mockRejectedValue(new Error(errorMessage));

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const order = await service.placeOrder('binance', 'BTC/USDT', 'market', 'buy', 100); // Large amount likely to fail

    expect(order).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error placing order on binance for BTC/USDT:',
      expect.any(Error)
    );
    consoleErrorSpy.mockRestore();
  });

  // --- closePosition Tests ---

  it('should close a long position successfully (sends market sell)', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const longPosition: Position = {
      symbol: 'BTC/USDT', // TradingPair (assuming string for simplicity in test)
      side: 'long',
      entryPrice: 50000,
      markPrice: 51000,
      amount: 1,         // Optional, but needed for assertion
      margin: 5000,      // Required
      pnl: 1000,         // Required
      leverage: 10,      // Required
      info: {}           // Required
    };
    const expectedCloseOrder = { id: 'closeOrder789', symbol: 'BTC/USDT', type: 'market', side: 'sell', amount: 1, info: {} };
    // biome-ignore lint/suspicious/noExplicitAny: Testing successful closePosition response format
    mockBinanceInstance.createOrder.mockResolvedValue(expectedCloseOrder as any);

    const service = new ExchangeService();
    const order = await service.closePosition('binance', longPosition);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('BTC/USDT', 'market', 'sell', 1);
    expect(order).toEqual(expectedCloseOrder);
  });

  it('should close a short position successfully (sends market buy)', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    // Using 'amount' instead of 'contracts' for variety
    const shortPosition: Position = { 
      symbol: 'ETH/USDT', 
      side: 'short', 
      amount: 1.5, 
      entryPrice: 50000, 
      markPrice: 51000, 
      leverage: 10, 
      margin: 5000, 
      pnl: 1000, 
      info: {} 
    };
    const expectedCloseOrder = { id: 'closeOrder101', symbol: 'ETH/USDT', type: 'market', side: 'buy', amount: 1.5, info: {} };
    // biome-ignore lint/suspicious/noExplicitAny: Testing successful closePosition (short) response format
    mockBinanceInstance.createOrder.mockResolvedValue(expectedCloseOrder as any);

    const service = new ExchangeService();
    const order = await service.closePosition('binance', shortPosition);

    expect(mockBinanceInstance.loadMarkets).toHaveBeenCalled();
    expect(mockBinanceInstance.createOrder).toHaveBeenCalledWith('ETH/USDT', 'market', 'buy', 1.5);
    expect(order).toEqual(expectedCloseOrder);
  });

  it('should return null and log error if closePosition returns invalid order data', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const longPosition: Position = {
      symbol: 'BTC/USDT', // TradingPair (assuming string for simplicity in test)
      side: 'long',
      entryPrice: 50000,
      markPrice: 51000,
      amount: 1,         // Optional, but needed for assertion
      margin: 5000,      // Required
      pnl: 1000,         // Required
      leverage: 10,      // Required
      info: {}           // Required
    };
    const invalidOrderData = { symbol: 'BTC/USDT' }; // No id
    // biome-ignore lint/suspicious/noExplicitAny: Testing invalid closePosition response (missing id)
    mockBinanceInstance.createOrder.mockResolvedValue(invalidOrderData as any);

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const order = await service.closePosition('binance', longPosition);

    expect(order).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Invalid order data received from binance:',
      invalidOrderData
    );
    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if closePosition encounters an error', async () => {
    const MockedBinanceConstructor = (ccxt as unknown as Record<string, Mock<() => MockExchangeInstance>>).binance;
    const mockBinanceInstance = MockedBinanceConstructor();
    const longPosition: Position = {
      symbol: 'BTC/USDT', // TradingPair (assuming string for simplicity in test)
      side: 'long',
      entryPrice: 50000,
      markPrice: 51000,
      amount: 1,         // Optional, but needed for assertion
      margin: 5000,      // Required
      pnl: 1000,         // Required
      leverage: 10,      // Required
      info: {}           // Required
    };
    const errorMessage = 'Position already closed';
    mockBinanceInstance.createOrder.mockRejectedValue(new Error(errorMessage));

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const service = new ExchangeService();
    const order = await service.closePosition('binance', longPosition);

    expect(order).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error closing position on binance for BTC/USDT:',
      expect.any(Error)
    );
    consoleErrorSpy.mockRestore();
  });

  it('should return null and log error if an invalid exchangeId is provided', async () => { 
    const service = new ExchangeService();
    const invalidExchangeId = 'invalidExchange';
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {}); 

    const balanceResult = await service.getBalance(invalidExchangeId, 'USDT'); 
    expect(balanceResult).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      `Error fetching balance for ${'USDT'} on ${invalidExchangeId}:`, 
      expect.any(Error) 
    );
    const balanceError = consoleErrorSpy.mock.calls[0][1] as Error;
    expect(balanceError.message).toContain(`Unsupported exchange: ${invalidExchangeId}`);

    const fundingRateResult = await service.getFundingRate(invalidExchangeId, 'BTC/USDT'); 
    expect(fundingRateResult).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      `Error fetching funding rate for ${'BTC/USDT'} on ${invalidExchangeId}:`, 
      expect.any(Error)
    );
    const fundingError = consoleErrorSpy.mock.calls[1][1] as Error;
    expect(fundingError.message).toContain(`Unsupported exchange: ${invalidExchangeId}`);

    consoleErrorSpy.mockRestore();
  });

});
