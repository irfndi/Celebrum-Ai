import { beforeEach, it, describe, vi, expect, Mock, afterEach } from 'vitest';
import { ExchangeService } from '../../src/services/exchangeService';
import { ExchangeId, TradingPairSymbol } from '../../src/types';
import type { Logger } from '../../src/utils/logger';
import { createMockExchangeService } from './exchangeService.test.helpers';
import { CustomError, APIError, NetworkError } from '../../src/utils/CustomError';
import type { Exchange as CcxtExchange } from 'ccxt';

// Import CCXT mock errors directly from the mock file
import { 
  BaseError,
  AuthenticationError,
  PermissionDenied,
  InvalidNonce,
  InsufficientFunds,
  InvalidOrder,
  RateLimitExceeded,
  DDoSProtection,
  RequestTimeout,
  NetworkError as CCXTNetworkError,
  ExchangeError
} from '../__mocks__/ccxt';

describe('ExchangeService Error Handling Tests', () => {
  let mockService: ReturnType<typeof createMockExchangeService>;
  let exchangeService: ExchangeService;
  let mockLogger: Logger;
  let mockExchangeInstance: CcxtExchange;

  beforeEach(() => {
    mockService = createMockExchangeService();
    exchangeService = mockService.exchangeService;
    mockLogger = mockService.mockLogger;
    mockExchangeInstance = mockService.singleMockExchangeInstance;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });
  
  describe('Error handling for core methods', () => {
    it('should handle errors in loadMarketsForExchange', async () => {
      // Setup mock to throw error
      (mockExchangeInstance.loadMarkets as Mock).mockRejectedValueOnce(
        new Error('Mock error loading markets')
      );
      
      // Execute the method
      const result = await exchangeService.loadMarketsForExchange('binance');
      
      // Verify expectations
      expect(result).toBeNull();
      expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error loading markets for binance'),
        expect.any(Error)
      );
    });
    
    it('should handle errors in getBalance', async () => {
      // Setup mock to throw error
      (mockExchangeInstance.fetchBalance as Mock).mockRejectedValueOnce(
        new Error('Mock error fetching balance')
      );
      
      // Execute the method
      const result = await exchangeService.getBalance('binance');
      
      // Verify expectations
      expect(result).toBeNull();
      expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error fetching balance'),
        expect.any(Error)
      );
    });
    
    it('should handle errors in getOpenOrders', async () => {
      // Setup mock to throw error
      (mockExchangeInstance.fetchOpenOrders as Mock).mockRejectedValueOnce(
        new Error('Mock error fetching open orders')
      );
      
      // Execute the method
      const result = await exchangeService.getOpenOrders('binance');
      
      // Verify expectations
      expect(result).toBeNull();
      expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error fetching open orders'),
        expect.any(Error)
      );
    });
    
    it('should handle errors in getOpenPositions', async () => {
      // Setup mock to throw error
      (mockExchangeInstance.fetchPositions as Mock).mockRejectedValueOnce(
        new Error('Mock error fetching positions')
      );
      
      // Execute the method
      const result = await exchangeService.getOpenPositions('binance');
      
      // Verify expectations
      expect(result).toBeNull();
      expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error fetching open positions'),
        expect.any(Error)
      );
    });
    
    it('should handle errors in setLeverage', async () => {
      // Setup mock to throw error
      (mockExchangeInstance.setLeverage as Mock).mockRejectedValueOnce(
        new Error('Mock error setting leverage')
      );
      
      // Execute the method
      const result = await exchangeService.setLeverage('binance', 'BTC/USDT', 10);
      
      // Verify expectations
      expect(result).toBeNull();
      expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error setting leverage'),
        expect.any(Error)
      );
    });
    
    it('should throw APIError for createOrder on exchange failure', async () => {
      // Setup a custom error
      const customError = new ExchangeError('Custom exchange error');
      (mockExchangeInstance.createOrder as Mock).mockRejectedValueOnce(customError);
      
      // Execute and verify
      await expect(exchangeService.createOrder(
        'binance',
        'BTC/USDT',
        'market',
        'buy',
        1,
        0
      )).rejects.toThrow(APIError);
    });
    
    it('should throw APIError for getTicker on authentication error', async () => {
      // Setup a custom auth error
      const authError = new AuthenticationError('Invalid API credentials');
      (mockExchangeInstance.fetchTicker as Mock).mockRejectedValueOnce(authError);
      
      // Execute and verify
      await expect(exchangeService.getTicker('binance', 'BTC/USDT'))
        .rejects.toThrow(APIError);
    });
    
    it('should throw NetworkError for getTicker on network issues', async () => {
      // Setup a network error
      const networkError = new CCXTNetworkError('Network connectivity issue');
      (mockExchangeInstance.fetchTicker as Mock).mockRejectedValueOnce(networkError);
      
      // Execute and verify
      await expect(exchangeService.getTicker('binance', 'BTC/USDT'))
        .rejects.toThrow(NetworkError);
    });
    
    it('should throw API error for invalid limit order price', async () => {
      await expect(exchangeService.createOrder(
        'binance',
        'BTC/USDT',
        'limit',
        'buy',
        1,
        undefined
      )).rejects.toThrow(APIError);
    });
  });
}); 