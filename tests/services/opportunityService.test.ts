import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { Mock, Mocked } from 'vitest';
import { OpportunityService } from '../../src/services/opportunityService';
import type { ExchangeId, TradingPairSymbol, ArbitrageOpportunity, FundingRateInfo, StructuredTradingPair } from '../../src/types'; 
import { ExchangeService } from '../../src/services/exchangeService';
import { TelegramService } from '../../src/services/telegramService'; // Import TelegramService

// Mock the ExchangeService
vi.mock('../../src/services/exchangeService');
vi.mock('../../src/services/telegramService'); // Mock TelegramService

// Mock Date.now() for consistent timestamps in tests
const MOCK_TIMESTAMP = Date.now(); // Example fixed timestamp
vi.setSystemTime(MOCK_TIMESTAMP);

describe('OpportunityService', () => {
  let opportunityService: OpportunityService;
  let mockExchangeService: Mocked<ExchangeService>; // Keep Mocked<T> for type safety
  let mockTelegramService: Mocked<TelegramService>; // Mocked TelegramService
  let mockLogger: {
      log: Mock;
      error: Mock;
      warn: Mock;
      info: Mock;
  };

  beforeEach(() => {
    // Reset mocks before each test
    vi.clearAllMocks();

    // Create a typed deep mocked instance using vi.mocked()
    // The second argument 'true' enables deep mocking if needed, though not strictly required here.
    mockExchangeService = vi.mocked(new ExchangeService(), true);
    // Mock Telegram configuration (adjust token and chatId as needed for tests)
    const mockTelegramConfig = {
      botToken: 'mock-token',
      chatId: 'mock-chat-id',
      logger: mockLogger // Include the logger in the config
    };
    // Pass the single config object to the mock constructor
    mockTelegramService = vi.mocked(new TelegramService(mockTelegramConfig), true); 
    // Default mock implementation for success case
    mockTelegramService.sendOpportunityNotification.mockResolvedValue(undefined); 

    // Create a fresh mock logger for each test
    mockLogger = {
        log: vi.fn(),
        error: vi.fn(),
        warn: vi.fn(),
        info: vi.fn(),
    };

    // Create OpportunityService with the mocked ExchangeService and TelegramService
    opportunityService = new OpportunityService(mockExchangeService, mockTelegramService, mockLogger);
  });

  it('should be defined', () => {
    expect(opportunityService).toBeDefined();
  });

  it('should find one opportunity when rate difference exceeds threshold', async () => {
    const exchangeIds: ExchangeId[] = ['binance', 'bybit'];
    const pairs: TradingPairSymbol[] = ['BTC/USDT'];
    const threshold = 0.0005; // 0.05%

    // Mock getFundingRate responses
    // Simulate Binance rate higher than Bybit
    mockExchangeService.getFundingRate
      .mockImplementation(async (exchangeId: string, pair: string): Promise<FundingRateInfo | null> => {
        if (pair !== 'BTC/USDT') return null;
        if (exchangeId === 'binance') {
          return { 
            info: { someData: 'from Binance' }, 
            symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' },
            timestamp: MOCK_TIMESTAMP,
            datetime: new Date(MOCK_TIMESTAMP).toISOString(),
            fundingRate: 0.0010, 
          };
        } 
        if (exchangeId === 'bybit') { 
          return { 
            info: { otherData: 'from Bybit' }, 
            symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' },
            timestamp: MOCK_TIMESTAMP,
            datetime: new Date(MOCK_TIMESTAMP).toISOString(),
            fundingRate: 0.0001, 
          };
        }
        return null;
      });

    const opportunities = await opportunityService.findOpportunities(exchangeIds, pairs, threshold);

    // Assertions
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledTimes(2); // Called for each exchange
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledWith('binance', 'BTC/USDT');
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledWith('bybit', 'BTC/USDT');

    expect(opportunities).toHaveLength(1);
    const opportunity = opportunities[0];
    expect(opportunity).toEqual<ArbitrageOpportunity>({
      pair: 'BTC/USDT',
      longExchange: 'bybit', // Lower rate
      shortExchange: 'binance', // Higher rate
      longRate: 0.0001,
      shortRate: 0.0010,
      rateDifference: 0.0009,
      timestamp: MOCK_TIMESTAMP,
    });
  });

  it('should find no opportunities when rate difference is below threshold', async () => {
    const exchangeIds: ExchangeId[] = ['binance', 'bybit'];
    const pairs: TradingPairSymbol[] = ['BTC/USDT'];
    const threshold = 0.001; // Higher threshold

    // Mock rates with difference 0.0009 (less than threshold 0.001)
    mockExchangeService.getFundingRate
      .mockImplementation(async (exchangeId: string, pair: string): Promise<FundingRateInfo | null> => {
        if (pair !== 'BTC/USDT') return null;
        if (exchangeId === 'binance') return { fundingRate: 0.0010, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' }; // Simplified mock
        if (exchangeId === 'bybit') return { fundingRate: 0.0001, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
        return null;
      });

    const opportunities = await opportunityService.findOpportunities(exchangeIds, pairs, threshold);

    expect(opportunities).toHaveLength(0);
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledTimes(2);
  });

  it('should handle multiple pairs and exchanges, finding multiple opportunities', async () => {
    const exchangeIds: ExchangeId[] = ['binance', 'bybit', 'kraken'];
    const pairs: TradingPairSymbol[] = ['BTC/USDT', 'ETH/USDT'];
    const threshold = 0.0005;

    // Mock rates
    mockExchangeService.getFundingRate
      .mockImplementation(async (exchangeId: string, pair: string): Promise<FundingRateInfo | null> => {
        // BTC: binance > bybit (opportunity), kraken null
        if (pair === 'BTC/USDT') {
          if (exchangeId === 'binance') return { fundingRate: 0.0010, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
          if (exchangeId === 'bybit') return { fundingRate: 0.0001, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
          if (exchangeId === 'kraken') return null; // Simulate fetch failure
        }
        // ETH: kraken > binance (opportunity), bybit low diff
        if (pair === 'ETH/USDT') {
          if (exchangeId === 'binance') return { fundingRate: 0.0002, info: {}, symbol: { base: 'ETH', quote: 'USDT', symbol: 'ETH/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
          if (exchangeId === 'bybit') return { fundingRate: 0.0001, info: {}, symbol: { base: 'ETH', quote: 'USDT', symbol: 'ETH/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' }; // Diff 0.0001 < threshold
          if (exchangeId === 'kraken') return { fundingRate: 0.0008, info: {}, symbol: { base: 'ETH', quote: 'USDT', symbol: 'ETH/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' }; 
        }
        return null;
      });

    const opportunities = await opportunityService.findOpportunities(exchangeIds, pairs, threshold);

    // Assertions
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledTimes(6); // 3 exchanges * 2 pairs
    expect(opportunities).toHaveLength(3); // Corrected expectation from 2 to 3

    // Check BTC opportunity (binance vs bybit)
    const btcOpp = opportunities.find(opp => opp.pair === 'BTC/USDT');
    expect(btcOpp).toBeDefined();
    expect(btcOpp).toEqual<ArbitrageOpportunity>({
      pair: 'BTC/USDT',
      longExchange: 'bybit',
      shortExchange: 'binance',
      longRate: 0.0001,
      shortRate: 0.0010,
      rateDifference: 0.0009,
      timestamp: MOCK_TIMESTAMP,
    });

    // Check ETH opportunity (kraken vs binance)
    const ethOpp = opportunities.find(opp => opp.pair === 'ETH/USDT' && opp.longExchange === 'binance' && opp.shortExchange === 'kraken');
    expect(ethOpp).toBeDefined();
    // Check properties individually, using toBeCloseTo for rateDifference
    expect(ethOpp?.pair).toBe('ETH/USDT');
    expect(ethOpp?.longExchange).toBe('binance');
    expect(ethOpp?.shortExchange).toBe('kraken');
    expect(ethOpp?.longRate).toBe(0.0002);
    expect(ethOpp?.shortRate).toBe(0.0008);
    expect(ethOpp?.rateDifference).toBeCloseTo(0.0006);
    expect(ethOpp?.timestamp).toBe(MOCK_TIMESTAMP);

    // Check ETH opportunity (bybit vs kraken)
    const ethOpp2 = opportunities.find(opp => opp.pair === 'ETH/USDT' && opp.longExchange === 'bybit' && opp.shortExchange === 'kraken');
    expect(ethOpp2).toBeDefined();
    // Check properties individually, using toBeCloseTo for rateDifference
    expect(ethOpp2?.pair).toBe('ETH/USDT');
    expect(ethOpp2?.longExchange).toBe('bybit');
    expect(ethOpp2?.shortExchange).toBe('kraken');
    expect(ethOpp2?.longRate).toBe(0.0001);
    expect(ethOpp2?.shortRate).toBe(0.0008);
    expect(ethOpp2?.rateDifference).toBeCloseTo(0.0007);
    expect(ethOpp2?.timestamp).toBe(MOCK_TIMESTAMP);
  });

  it('should correctly ignore pairs where one or more exchanges return null rates', async () => {
    const exchangeIds: ExchangeId[] = ['binance', 'bybit'];
    const pairs: TradingPairSymbol[] = ['BTC/USDT'];
    const threshold = 0.0005;

    // Mock getFundingRate responses - bybit returns null
    mockExchangeService.getFundingRate
      .mockImplementation(async (exchangeId: string, pair: string): Promise<FundingRateInfo | null> => {
        if (pair !== 'BTC/USDT') return null;
        if (exchangeId === 'binance') return { fundingRate: 0.001, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
        if (exchangeId === 'bybit') return null; // Bybit fails
        return null;
      });

    const opportunities = await opportunityService.findOpportunities(exchangeIds, pairs, threshold);

    // Assertions
    expect(opportunities).toHaveLength(0); // No opportunity because bybit rate is null
    expect(mockExchangeService.getFundingRate).toHaveBeenCalledTimes(2);
  });

  it('should log an error if sending Telegram notification fails', async () => {
    // Arrange
    const exchangeIds: ExchangeId[] = ['binance', 'bybit'];
    const pairs: TradingPairSymbol[] = ['BTC/USDT'];
    const threshold = 0.0005;

    // Mock getFundingRate to return rates that create an opportunity
    mockExchangeService.getFundingRate.mockImplementation(async (exchangeId: string, pair: string): Promise<FundingRateInfo | null> => {
      if (pair === 'BTC/USDT') {
        if (exchangeId === 'binance') return { fundingRate: 0.001, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
        if (exchangeId === 'bybit') return { fundingRate: 0.0001, info: {}, symbol: { base: 'BTC', quote: 'USDT', symbol: 'BTC/USDT' }, timestamp: MOCK_TIMESTAMP, datetime: '' };
      }
      return null;
    });

    // Mock sendOpportunityNotification to throw an error
    const telegramError = new Error('Telegram API Error');
    mockTelegramService.sendOpportunityNotification.mockRejectedValue(telegramError);

    // Act
    const opportunities = await opportunityService.findOpportunities(exchangeIds, pairs, threshold);

    // Allow pending promises (like the .catch() in findOpportunities) to settle
    await new Promise(resolve => setTimeout(resolve, 0));

    // Assert
    expect(opportunities).toHaveLength(1); // Opportunity should still be found
    expect(mockTelegramService.sendOpportunityNotification).toHaveBeenCalledTimes(1); // Attempted to send
    expect(mockLogger.error).toHaveBeenCalledTimes(1); // Error should be logged
    expect(mockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining('Error sending Telegram notification for BTC/USDT:'),
        telegramError
    );
  });
});
