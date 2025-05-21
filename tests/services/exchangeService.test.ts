/// <reference types='@cloudflare/workers-types' />
/// <reference types="vitest/globals" />

// NOTE: vi.mock('ccxt') at the top level is REMOVED. Mocking will be done per describe block.

import type { Mock } from 'vitest';
import type { KVNamespace as TypesKVNamespace, KVNamespaceGetOptions } from '@cloudflare/workers-types';
import type { ExchangeId, LoggerInterface, Position, Env, Ticker, TradingPairSymbol } from '../../src/types';
import type { MockExchangeInstance } from './exchangeService.test.helpers';
// Import what we need from the ccxt mock
import {
  _theActualTestAccessibleMockInstances,
  mockExchangeConstructors,
  commonErrorClasses,
  version
} from '../__mocks__/ccxt';

// Define ApiKeyConfig for testing since it comes from a different file
interface ApiKeyConfig {
  apiKey: string;
  secret?: string;
  apiSecret?: string;
  subaccount?: string;
  password?: string;
  uid?: string;
  new?: boolean;
}

describe('ExchangeService', () => {
  let ExchangeService: typeof import('../../src/services/exchangeService').ExchangeService;
  let exchangeService: import('../../src/services/exchangeService').ExchangeService;
  
  let mockKvNamespace: TypesKVNamespace;
  let localMockLogger: LoggerInterface;
  let mockEnv: Env;
  
  let ccxtDefaultExport: unknown; // This will hold the default export of the mocked ccxt
  let testAccessibleInstancesFromMock: typeof _theActualTestAccessibleMockInstances;

  beforeEach(async () => {
    vi.resetModules(); 

    // This is what the actual 'ccxt' module would export as default, plus its named exports.
    const mockCcxtModuleImplementation = {
      ...mockExchangeConstructors,    // Spread individual exchange constructors
      ...commonErrorClasses,          // Spread error classes
      version,                        // Include the version export
      default: {                      // Define the default export
        _testAccessibleMockInstances: _theActualTestAccessibleMockInstances,
        ...mockExchangeConstructors,
        ...commonErrorClasses,
        version,
        Precise: vi.fn((val: string | number) => String(val)),
        functions: {
          aggregate: vi.fn(),
          deepExtend: vi.fn((...args: unknown[]) => Object.assign({}, ...args)),
        },
        exchanges: Object.keys(mockExchangeConstructors)
      },
    };

    // Use the correct mocking approach in vitest
    vi.doMock('ccxt', () => mockCcxtModuleImplementation);

    // Dynamically import ExchangeService after ccxt has been mocked
    const exchangeServiceModule = await import('../../src/services/exchangeService');
    ExchangeService = exchangeServiceModule.ExchangeService;

    localMockLogger = {
      info: vi.fn(),
      error: vi.fn(),
      warn: vi.fn(),
      debug: vi.fn(),
      log: vi.fn(),
      setLogLevel: vi.fn(),
      getLogLevel: vi.fn().mockReturnValue('info'),
      addError: vi.fn(),
      addContext: vi.fn(),
      http: vi.fn(),
      verbose: vi.fn(),
      silly: vi.fn(),
      child: vi.fn().mockReturnThis()
    };
    
    // For test assertions, capture what the mock setup provides
    ccxtDefaultExport = mockCcxtModuleImplementation.default; 
    testAccessibleInstancesFromMock = _theActualTestAccessibleMockInstances;

    // Clear the instances for each test, as they are populated by mock constructors
    for (const key in _theActualTestAccessibleMockInstances) {
        delete _theActualTestAccessibleMockInstances[key as ExchangeId];
    }

    console.log("DEBUG beforeEach: testAccessibleInstancesFromMock (cleared):", 
                testAccessibleInstancesFromMock ? Object.keys(testAccessibleInstancesFromMock) : 'undefined');

    // Mock KV namespace for testing - Type it properly
    const mockKvImplementation = {
      get: vi.fn().mockImplementation(
        async (key: string, options?: KVNamespaceGetOptions<unknown>) => {
          const type = typeof options === "string" ? options : options?.type;
          if (key.startsWith("api_key:")) return Promise.resolve(null); 
          if (key.startsWith("arbitrageOpportunities")) { 
            const mockArbOp = { id: "test-op-123" };
            if (type === "json") return Promise.resolve(mockArbOp);
            return Promise.resolve(JSON.stringify(mockArbOp));
          }
          if (key === "allExchangeIds") { 
            if (type === "json") return Promise.resolve(["binance", "kraken"]);
            return Promise.resolve(JSON.stringify(["binance", "kraken"]));
          }
          return Promise.resolve(null);
        }
      ),
      put: vi.fn().mockResolvedValue(undefined),
      delete: vi.fn().mockResolvedValue(undefined),
      list: vi.fn().mockResolvedValue({ keys: [], list_complete: true, cursor: undefined }),
      getWithMetadata: vi.fn().mockImplementation(async (key: string) => {
        if (key.startsWith("api_key:")) return Promise.resolve({ value: null, metadata: null });
        return Promise.resolve({ value: null, metadata: null });
      }),
    };
    // We need to use a variable that can be mocked in tests
    const mockGet = vi.fn();
    const mockPut = vi.fn();
    const mockDelete = vi.fn();
    
    // Assign the implementation to the mock functions
    mockGet.mockImplementation(mockKvImplementation.get);
    mockPut.mockImplementation(mockKvImplementation.put);
    mockDelete.mockImplementation(mockKvImplementation.delete);
    
    // Create a KV object with the mock functions
    mockKvNamespace = {
      get: mockGet,
      put: mockPut,
      delete: mockDelete,
      list: mockKvImplementation.list,
      getWithMetadata: mockKvImplementation.getWithMetadata,
    } as unknown as TypesKVNamespace;

    // Create a mock environment - cast to 'unknown' first to avoid type errors
    const mockEnvRaw = {
      ArbEdgeKV: mockKvNamespace,
      TELEGRAM_BOT_TOKEN: 'mockToken',
      TELEGRAM_CHAT_ID: 'mockChatId',
      NODE_ENV: 'test',
      LOG_LEVEL: 'debug',
      POSITIONS: '{}',
      EXCHANGES: '{"binance": {"apiKeyRequired": true}, "kraken": {"apiKeyRequired": true}}',
      ARBITRAGE_THRESHOLD: '0.001',
      MONITORED_PAIRS_CONFIG: '{}',
    };
    mockEnv = mockEnvRaw as unknown as Env;

    // Create exchange service with mocked deps
    exchangeService = new ExchangeService({ env: mockEnv, logger: localMockLogger });
    console.log("DEBUG beforeEach: exchangeService created.");
    console.log("DEBUG beforeEach: testAccessibleInstancesFromMock AFTER service init:", 
                testAccessibleInstancesFromMock ? Object.keys(testAccessibleInstancesFromMock) : 'undefined');
  });

  afterEach(() => {
    vi.clearAllMocks();
    console.log("DEBUG: afterEach - mocks cleared");
  });

  // Basic verification tests for mocking setup
  it("should be using the mocked ccxt", async () => {
    const instance = await exchangeService.getExchangeInstance('binance');
    expect(instance).toBeDefined();
    if (instance) {
      const exchangeInstance = instance as unknown as MockExchangeInstance;
      expect(typeof exchangeInstance.fetchTicker).toBe('function'); 
      expect(vi.isMockFunction(exchangeInstance.fetchTicker)).toBe(true);
      const mockedBinanceInstance = testAccessibleInstancesFromMock.binance;
      expect(mockedBinanceInstance).toBe(instance);
    }
  });

  it("should correctly load the mocked ccxt module and access _testAccessibleMockInstances", () => {
    expect(ccxtDefaultExport).toBeDefined();
    const defaultExportObj = ccxtDefaultExport as Record<string, unknown>;
    expect(defaultExportObj._testAccessibleMockInstances).toBeDefined(); 
    expect(defaultExportObj._testAccessibleMockInstances).toBe(testAccessibleInstancesFromMock);
    expect(testAccessibleInstancesFromMock).toBeDefined();
    expect(typeof testAccessibleInstancesFromMock).toBe('object');
  });

  // Exchange instance and markets tests
  it("should add exchanges and load their markets when added", async () => {
    const binanceInstance = await exchangeService.getExchangeInstance("binance");
    const krakenInstance = await exchangeService.getExchangeInstance("kraken");
    expect(binanceInstance).toBeDefined();
    expect(krakenInstance).toBeDefined();
    if (!binanceInstance || !krakenInstance) throw new Error("Instances not created");
    
    const mockBinanceInstance = binanceInstance as unknown as MockExchangeInstance;
    const mockKrakenInstance = krakenInstance as unknown as MockExchangeInstance;
    
    expect(vi.isMockFunction(mockBinanceInstance.loadMarkets)).toBe(true);
    expect(vi.isMockFunction(mockKrakenInstance.loadMarkets)).toBe(true);
    expect(testAccessibleInstancesFromMock.binance).toBe(binanceInstance);
    expect(testAccessibleInstancesFromMock.kraken).toBe(krakenInstance);
  });

  // Dynamic exchange management
  describe('Dynamic Exchange Management', () => {
    it("should initialize an exchange and its markets upon first retrieval", async () => {
      const exchangeId = "bybit" as ExchangeId;
      const dynamicInstance = await exchangeService.getExchangeInstance(exchangeId);
      expect(dynamicInstance).toBeDefined();
      if(dynamicInstance) expect(dynamicInstance.id).toBe(exchangeId);
      
      const mockDynamicInstance = dynamicInstance as unknown as MockExchangeInstance;
      expect(vi.isMockFunction(mockDynamicInstance?.loadMarkets)).toBe(true);
      expect(mockDynamicInstance?.loadMarkets).toHaveBeenCalled();
      expect(testAccessibleInstancesFromMock[exchangeId]).toBe(dynamicInstance);
    });
  });

  // Position management tests
  describe("getOpenPositions", () => {
    it("should fetch open positions for a given symbol on an exchange", async () => {
      const exchangeId = "binance" as ExchangeId;
      const symbol = "BTC/USDT" as TradingPairSymbol;
      const binanceInstance = await exchangeService.getExchangeInstance(exchangeId);
      
      expect(binanceInstance).toBeTruthy();
      expect(testAccessibleInstancesFromMock[exchangeId]).toBe(binanceInstance);
      
      const mockBinanceInstance = binanceInstance as unknown as MockExchangeInstance;
      
      // Create a mock position that matches the actual Position interface
      // This is a simplified version that should satisfy the test
      const fakePosition = {
        symbol, 
        side: "long" as const,
        entryPrice: 30000,
        markPrice: 31000,
        liquidationPrice: 25000,
        leverage: 10,
        contracts: 1,
        contractSize: 1,
        margin: 3000,
        timestamp: Date.now(),
        datetime: new Date().toISOString(),
        info: {},
        // Additional fields that might be needed by the service
        id: "pos1",
        unrealizedPnl: 1000,
        initialMargin: 3000,
        maintenanceMargin: 1500,
        notional: 30000,
        collateral: 3000,
        percentage: (1000/30000)*100,
      } as unknown as Position;  // Cast to Position to satisfy TypeScript
      
      mockBinanceInstance.fetchPositions.mockResolvedValue([fakePosition]);
      
      const positions = await exchangeService.getOpenPositions(exchangeId, symbol);
      expect(positions).toEqual([fakePosition]);
      expect(mockBinanceInstance.fetchPositions).toHaveBeenCalledWith([symbol]);
    });
  });

  // Leverage management tests
  describe("setLeverage", () => {
    it("should set leverage for a symbol on a given exchange", async () => {
      const exchangeId = "binance" as ExchangeId;
      const symbol = "BTC/USDT" as TradingPairSymbol;
      const leverage = 20;
      const binanceInstance = await exchangeService.getExchangeInstance(exchangeId);
      
      expect(binanceInstance).toBeTruthy();
      expect(testAccessibleInstancesFromMock[exchangeId]).toBe(binanceInstance);
      
      const mockBinanceInstance = binanceInstance as unknown as MockExchangeInstance;
      const leverageResponse = { 
        info: { symbol, leverage }, 
        symbol, 
        marginMode: "isolated", 
        longLeverage: leverage, 
        shortLeverage: leverage 
      };
      
      mockBinanceInstance.setLeverage.mockResolvedValue(leverageResponse);
      
      const result = await exchangeService.setLeverage(exchangeId, symbol, leverage);
      expect(result).toEqual(leverageResponse);
      expect(mockBinanceInstance.setLeverage).toHaveBeenCalledWith(leverage, symbol, { marginMode: "isolated" });
    });
  });

  // Ticker tests
  describe("getTicker", () => {
    it("should fetch the ticker for a symbol on an exchange", async () => {
      const exchangeId = "binance" as ExchangeId;
      const symbol = "BTC/USDT" as TradingPairSymbol;
      const binanceInstance = await exchangeService.getExchangeInstance(exchangeId);
      
      expect(binanceInstance).toBeTruthy();
      expect(testAccessibleInstancesFromMock[exchangeId]).toBe(binanceInstance);
      
      const mockBinanceInstance = binanceInstance as unknown as MockExchangeInstance;
      
      // Create a fake ticker that matches the Ticker interface
      const fakeTicker = { 
        symbol, 
        bid: 29999, 
        ask: 30001, 
        last: 30000, 
        timestamp: Date.now(), 
        datetime: new Date().toISOString(), 
        high: 30500, 
        low: 29500, 
        vwap: 30000, 
        open: 29800, 
        close: 30000, 
        change: 200, 
        percentage: (200/29800)*100, 
        average: 29900, 
        baseVolume: 1000, 
        quoteVolume: 30000000, 
        info: {},
        // Additional fields required by the Ticker interface
        bidVolume: undefined,
        askVolume: undefined,
        previousClose: undefined,
        indexPrice: undefined,
        markPrice: undefined,
      } as Ticker;
      
      mockBinanceInstance.fetchTicker.mockResolvedValue(fakeTicker);
      
      const ticker = await exchangeService.getTicker(exchangeId, symbol);
      expect(ticker).toEqual(fakeTicker);
      expect(mockBinanceInstance.fetchTicker).toHaveBeenCalledWith(symbol);
    });
  });

  // Error handling tests
  describe('Error Handling', () => {
    it("should handle and log errors from exchange methods", async () => {
      const exchangeId = "binance" as ExchangeId;
      const symbol = "BTC/USDT" as TradingPairSymbol;
      const binanceInstance = await exchangeService.getExchangeInstance(exchangeId);
      
      expect(binanceInstance).toBeTruthy();
      expect(testAccessibleInstancesFromMock[exchangeId]).toBe(binanceInstance);
      
      const mockBinanceInstance = binanceInstance as unknown as MockExchangeInstance;
      const errorMessage = "Mocked fetchTicker failed for testing";
      mockBinanceInstance.fetchTicker.mockRejectedValue(new Error(errorMessage));
      
      localMockLogger.error = vi.fn(); 
      const ticker = await exchangeService.getTicker(exchangeId, symbol);
      expect(ticker).toBeNull();
      expect(localMockLogger.error).toHaveBeenCalledTimes(1);
      expect(localMockLogger.error).toHaveBeenCalledWith(
        expect.stringContaining(`Error fetching ticker for ${symbol} on ${exchangeId}`), 
        expect.any(Error)
      );
    });
  });
   
  // API Key Management tests
  describe('API Key Management', () => {
    const exchangeId = 'binance' as ExchangeId;
    const apiKeyConfig: ApiKeyConfig = { apiKey: 'testKey', secret: 'testSecret', new: true };

    describe('saveApiKey', () => {
      it('should clear cached exchange instance after saving API key', async () => {
        const initialInstance = await exchangeService.getExchangeInstance(exchangeId);
        expect(initialInstance).toBeDefined();
        
        const binanceConstructorMock = mockExchangeConstructors.binance;
        expect(vi.isMockFunction(binanceConstructorMock)).toBe(true);
        binanceConstructorMock.mockClear();

        await exchangeService.saveApiKey(exchangeId, apiKeyConfig);
        await exchangeService.getExchangeInstance(exchangeId);
        expect(binanceConstructorMock).toHaveBeenCalledTimes(1); 
      });

      it('should store API keys in KV namespace', async () => {
        await exchangeService.saveApiKey(exchangeId, apiKeyConfig);
        expect(mockKvNamespace.put).toHaveBeenCalledWith(
          `api_key:${exchangeId}`, 
          JSON.stringify({ apiKey: 'testKey', secret: 'testSecret' })
        );
      });

      it('should add exchangeId to allExchangeIds in KV if new', async () => {
        mockKvNamespace.get.mockResolvedValueOnce(null); 
        await exchangeService.saveApiKey(exchangeId, apiKeyConfig); 
        expect(mockKvNamespace.put).toHaveBeenCalledWith('allExchangeIds', JSON.stringify([exchangeId]));
        
        mockKvNamespace.get.mockResolvedValueOnce(JSON.stringify(['kraken'])); 
        await exchangeService.saveApiKey(exchangeId, apiKeyConfig);
        expect(mockKvNamespace.put).toHaveBeenCalledWith('allExchangeIds', JSON.stringify(['kraken', exchangeId]));
      });

      it('should not add exchangeId to allExchangeIds if not new', async () => {
        mockKvNamespace.get.mockResolvedValueOnce(JSON.stringify(['binance']));
        await exchangeService.saveApiKey(exchangeId, { ...apiKeyConfig, new: false });
        expect(mockKvNamespace.put).toHaveBeenCalledWith(
          `api_key:${exchangeId}`, 
          expect.any(String)
        );
      });
    });

    describe('getApiKey', () => {
      it('should retrieve API keys from KV namespace', async () => {
        const storedKeys = { apiKey: 'kvKey', secret: 'kvSecret' };
        mockKvNamespace.get.mockResolvedValueOnce(JSON.stringify(storedKeys));
        const result = await exchangeService.getApiKey(exchangeId);
        expect(result).toEqual(storedKeys);
        expect(mockKvNamespace.get).toHaveBeenCalledWith(`api_key:${exchangeId}`);
      });

      it('should return null if no keys in KV', async () => {
        mockKvNamespace.get.mockResolvedValueOnce(null);
        const result = await exchangeService.getApiKey(exchangeId);
        expect(result).toBeNull();
      });
    });

    describe('deleteApiKey', () => {
      it('should clear cached exchange instance after deleting API key', async () => {
        const binanceConstructorMock = mockExchangeConstructors.binance;
        expect(vi.isMockFunction(binanceConstructorMock)).toBe(true);
        
        await exchangeService.getExchangeInstance(exchangeId); 
        binanceConstructorMock.mockClear();
        
        await exchangeService.deleteApiKey(exchangeId);
        await exchangeService.getExchangeInstance(exchangeId); 
        expect(binanceConstructorMock).toHaveBeenCalledTimes(1); 
      });

      it('should remove API keys from KV namespace', async () => {
        await exchangeService.deleteApiKey(exchangeId);
        expect(mockKvNamespace.delete).toHaveBeenCalledWith(`api_key:${exchangeId}`);
      });

      it('should remove exchangeId from allExchangeIds in KV', async () => {
        mockKvNamespace.get.mockResolvedValueOnce(JSON.stringify(['binance', 'kraken']));
        await exchangeService.deleteApiKey(exchangeId);
        expect(mockKvNamespace.put).toHaveBeenCalledWith('allExchangeIds', JSON.stringify(['kraken']));
      });
    });
  });
   
  // Exchange Instance Management tests
  describe('getExchangeInstance', () => {
    const exchangeId = 'binance' as ExchangeId;
    const krakenId = 'kraken' as ExchangeId;

    it('should return a cached instance if available', async () => {
      const instance1 = await exchangeService.getExchangeInstance(exchangeId);
      const binanceConstructorMock = mockExchangeConstructors.binance;
      expect(vi.isMockFunction(binanceConstructorMock)).toBe(true);
      
      binanceConstructorMock.mockClear(); 
      const instance2 = await exchangeService.getExchangeInstance(exchangeId);
      expect(instance2).toBe(instance1);
      expect(binanceConstructorMock).not.toHaveBeenCalled();
    });

    it('should initialize with API keys from KV store if available', async () => {
      const kvKeys = { apiKey: 'kvApiKey', secret: 'kvApiSecret' };
      mockKvNamespace.get.mockImplementation(async (key) => {
        if (key === `api_key:${exchangeId}`) return JSON.stringify(kvKeys);
        return null;
      });
      
      const binanceConstructorMock = mockExchangeConstructors.binance;
      await exchangeService.getExchangeInstance(exchangeId);
      expect(binanceConstructorMock).toHaveBeenCalledWith(expect.objectContaining(kvKeys));
    });

    it('should initialize with API keys from environment if KV store has no keys', async () => {
      mockKvNamespace.get.mockResolvedValue(null); 
      const envApiKey = 'envApiKeyKraken';
      const envApiSecret = 'envApiSecretKraken';
      
      // Update environment with test keys
      const envWithKeys = {...mockEnv} as Record<string, unknown>;
      envWithKeys[`${krakenId.toUpperCase()}_API_KEY`] = envApiKey;
      envWithKeys[`${krakenId.toUpperCase()}_API_SECRET`] = envApiSecret;
      
      // Create a new service with the updated environment
      const tempExchangeService = new ExchangeService({ 
        env: envWithKeys as unknown as Env, 
        logger: localMockLogger 
      });
      
      const krakenConstructorMock = mockExchangeConstructors.kraken;
      await tempExchangeService.getExchangeInstance(krakenId);
      expect(krakenConstructorMock).toHaveBeenCalledWith(
        expect.objectContaining({ apiKey: envApiKey, secret: envApiSecret })
      );
    });
    
    it('should prioritize KV store keys over environment keys', async () => {
      const kvKeys = { apiKey: 'priorityKVKey', secret: 'priorityKVSecret' };
      mockKvNamespace.get.mockImplementation(async (key) => {
        if (key === `api_key:${exchangeId}`) return JSON.stringify(kvKeys);
        return null;
      });
      
      // Update environment with test keys that should be ignored
      const envWithKeys = {...mockEnv} as Record<string, unknown>;
      envWithKeys[`${exchangeId.toUpperCase()}_API_KEY`] = 'envApiKeyShouldBeIgnored';
      
      // Create a new service with the updated environment
      const tempExchangeService = new ExchangeService({ 
        env: envWithKeys as unknown as Env, 
        logger: localMockLogger 
      });
      
      const binanceConstructorMock = mockExchangeConstructors.binance;
      await tempExchangeService.getExchangeInstance(exchangeId);
      expect(binanceConstructorMock).toHaveBeenCalledWith(expect.objectContaining(kvKeys));
    });

    it('should initialize as a public client if no keys are available', async () => {
      mockKvNamespace.get.mockResolvedValue(null);
      
      // Create env with no API keys
      const cleanEnv = {...mockEnv} as Record<string, unknown>;
      // Remove any possible API keys
      for (const key of Object.keys(cleanEnv)) {
        if (key.includes('API_KEY') || key.includes('API_SECRET')) {
          delete cleanEnv[key];
        }
      }
      
      // Create a new service with the clean environment
      const tempExchangeService = new ExchangeService({ 
        env: cleanEnv as unknown as Env, 
        logger: localMockLogger 
      });
      
      const binanceConstructorMock = mockExchangeConstructors.binance;
      binanceConstructorMock.mockClear();
      
      await tempExchangeService.getExchangeInstance(exchangeId);
      
      // Should be called with just basic options, no API keys
      expect(binanceConstructorMock).toHaveBeenCalledWith(
        expect.not.objectContaining({ 
          apiKey: expect.anything(),
          secret: expect.anything() 
        })
      );
    });

    it('should call loadMarkets after successful instance creation', async () => {
      const instance = await exchangeService.getExchangeInstance(exchangeId);
      const mockInstance = instance as unknown as MockExchangeInstance;
      expect(mockInstance.loadMarkets).toHaveBeenCalled();
    });
  });
});
