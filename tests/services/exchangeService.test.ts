/// <reference types='@cloudflare/workers-types' />
/// <reference types="vitest/globals" />
import ccxt from "ccxt";
import type {
  ExchangeId,
  TradingPairSymbol,
  PositionSide,
  FundingRateInfo,
  ArbitrageOpportunity,
  Balances,
  Balance,
  Position,
  Order,
  Market,
  OrderSide,
  OrderType,
  Ticker,
  OHLCV,
  LoggerInterface,
  OrderBook,
  Trade,
  CCXTTradingFees,
  CCXTTradingFeeInterface,
} from "../../src/types";
import type { Exchange as CCXTExchange, Fee as CCXTFee } from "ccxt";
import { MOCK_MARKET_DEFAULTS } from "../mocks/marketMocks";
import {
  describe,
  it,
  expect,
  vi,
  beforeEach,
  afterEach,
  type Mock,
  type Mocked,
} from "vitest";
import { ExchangeService } from "../../src/services/exchangeService";

interface Leverage {
  info: Record<string, any>;
  symbol: string;
  marginMode?: "cross" | "isolated" | string;
  longLeverage: number;
  shortLeverage: number;
}

import type {
  Env,
  KVNamespace,
  KVNamespaceGetOptions,
  KVNamespaceListOptions,
  KVNamespaceListResult,
  KVNamespaceGetWithMetadataResult,
} from "@cloudflare/workers-types";

vi.mock("ccxt", async (importOriginal: () => Promise<typeof ccxt>) => {
  const originalCcxtModule = await importOriginal();
  const testHelpers = await import("./exchangeService.test.helpers");

  let factoryScopedSingletonInstances:
    | Record<ExchangeId, testHelpers.MockExchangeInstance>
    | undefined;

  const initializeMockInstancesIfNeeded = () => {
    if (!factoryScopedSingletonInstances) {
      const tempInstances: Partial<
        Record<ExchangeId, testHelpers.MockExchangeInstance>
      > = {};
      for (const id of testHelpers.ALL_MOCK_EXCHANGE_IDS) {
        tempInstances[id] = testHelpers.createMockInstance(id);
      }
      factoryScopedSingletonInstances = tempInstances as Record<
        ExchangeId,
        testHelpers.MockExchangeInstance
      >;
    }
  };

  // Factory function for creating mock exchange "instances"
  const createMockCcxtInstance = (exchangeId: ExchangeId, _options?: any) => {
    initializeMockInstancesIfNeeded();
    const mockInstanceData = factoryScopedSingletonInstances![exchangeId];
    if (!mockInstanceData) {
      throw new Error(
        `Mock instance data for ${exchangeId} not found in factory-scoped singleton.`
      );
    }
    // Return a new object that simulates a ccxt instance
    return {
      id: exchangeId, // Ensure 'id' is present, as ccxt instances have it
      ...mockInstanceData, // Spread all mocked methods and properties
      // If options are passed and need to be stored or used (e.g. apiKey), handle here
      // For instance:
      // apiKey: _options?.apiKey,
      // rateLimit: _options?.rateLimit || 2000, // Default rateLimit
      // verbose: _options?.verbose || false,
      // ... any other properties ccxt instances might have from options
    };
  };

  // The base "Exchange" class mock can be a function that throws or returns a generic base mock
  const MockBaseExchange = (_options?: any) => {
    // This could return a very generic mock or throw if direct instantiation of base Exchange is not expected
    // For now, let's make it similar to specific instances but with a generic ID or warning
    console.warn(
      "Mocked ccxt.Exchange base class instantiated. This might not be fully functional."
    );
    return {
      id: "mockBaseExchange",
      version: "mocked",
      rateLimit: 2000,
      // ...other common minimal properties
      ...testHelpers.createMockInstance("generic" as ExchangeId), // Use a generic mock if available or minimal
    };
  };

  const commonExports = {
    Exchange: MockBaseExchange, // Mock for the base Exchange class
    // Each exchange ID points to a function that will be called with 'new'
    // This function then returns our pre-configured mock object.
    binance: function (...args: any[]) {
      return createMockCcxtInstance("binance", args[0]);
    },
    bybit: function (...args: any[]) {
      return createMockCcxtInstance("bybit", args[0]);
    },
    bitget: function (...args: any[]) {
      return createMockCcxtInstance("bitget", args[0]);
    },
    kraken: function (...args: any[]) {
      return createMockCcxtInstance("kraken", args[0]);
    },
    mexc: function (...args: any[]) {
      return createMockCcxtInstance("mexc", args[0]);
    },
    okx: function (...args: any[]) {
      return createMockCcxtInstance("okx", args[0]);
    },
    bingx: function (...args: any[]) {
      return createMockCcxtInstance("bingx", args[0]);
    },

    pro: new Proxy(
      {},
      {
        get: (target, propKey) => {
          const exchangeId = String(propKey).toLowerCase() as ExchangeId;
          // Ensure factoryScopedSingletonInstances is initialized to check against its keys
          initializeMockInstancesIfNeeded();
          if (
            factoryScopedSingletonInstances &&
            factoryScopedSingletonInstances[exchangeId]
          ) {
            // For pro exchanges, return a function that produces the mock instance
            return function (...args: any[]) {
              return createMockCcxtInstance(exchangeId, args[0]);
            };
          }
          if (
            originalCcxtModule.pro &&
            originalCcxtModule.pro.hasOwnProperty(exchangeId) &&
            typeof (originalCcxtModule.pro as any)[exchangeId] === "function"
          ) {
            return (originalCcxtModule.pro as any)[exchangeId]; // Fallback to original if specific pro function exists
          }
          console.warn(
            `ccxt.pro mock: Exchange class '${exchangeId}' not explicitly mocked. Returning undefined.`
          );
          return undefined;
        },
      }
    ),
    get _testAccessibleMockInstances() {
      initializeMockInstancesIfNeeded();
      return factoryScopedSingletonInstances;
    },
  };

  return {
    ...originalCcxtModule, // Spread original ccxt for any unmocked parts
    ...commonExports, // Override with our mocks
    default: {
      // Ensure default export is also fully mocked
      ...originalCcxtModule,
      ...commonExports,
    },
  };
});

interface TestHelperMockedCcxtModule extends Readonly<typeof ccxt> {
  _testAccessibleMockInstances?: Record<
    ExchangeId,
    import("./exchangeService.test.helpers").MockExchangeInstance
  >;
}

let testAccessibleMockInstances:
  | Record<
      ExchangeId,
      import("./exchangeService.test.helpers").MockExchangeInstance
    >
  | undefined;
let mockEnv: Env;

describe("ExchangeService", () => {
  const logger: LoggerInterface = {
    log: vi.fn(console.log),
    error: vi.fn(console.error),
    warn: vi.fn(console.warn),
    info: vi.fn(console.info),
    debug: vi.fn(console.debug),
    addContext: vi.fn(),
    addError: vi.fn(),
  };

  beforeEach(async () => {
    const mockedCcxtModule = (await import(
      "ccxt"
    )) as unknown as TestHelperMockedCcxtModule;
    const ccxtMocks = mockedCcxtModule._testAccessibleMockInstances;
    if (ccxtMocks === undefined) {
      throw new Error(
        "_testAccessibleMockInstances was unexpectedly undefined during test setup. This indicates a problem with the mock initialization."
      );
    }
    testAccessibleMockInstances = ccxtMocks;

    const { MOCK_BINANCE_BALANCES_FACTORY } = await import(
      "./exchangeService.test.helpers"
    );

    mockEnv = {
      ArbEdgeKV: {
        get: vi
          .fn()
          .mockImplementation(
            (
              key: string,
              options?:
                | KVNamespaceGetOptions
                | "text"
                | "json"
                | "arrayBuffer"
                | "stream"
            ) => {
              const type =
                typeof options === "string" ? options : options?.type;
              if (key.startsWith("arbitrageOpportunities")) {
                const mockArbOp: ArbitrageOpportunity = {
                  id: "test-op-123",
                  exchangePair1: { exchange: "binance", symbol: "BTC/USDT" },
                  exchangePair2: { exchange: "kraken", symbol: "BTC/USD" },
                  profitPercentage: 1.5,
                  timestamp: Date.now(),
                  status: "open",
                  potentialGain: { amount: 15, currency: "USDT" },
                  paths: [],
                };
                if (type === "json") return Promise.resolve(mockArbOp);
                return Promise.resolve(JSON.stringify(mockArbOp));
              }
              if (key.startsWith("userConfig:")) {
                const mockUserConfig = {
                  apiKey: "test-key",
                  apiSecret: "test-secret",
                };
                if (type === "json") return Promise.resolve(mockUserConfig);
                return Promise.resolve(JSON.stringify(mockUserConfig));
              }
              if (key === "allExchangeIds") {
                if (type === "json")
                  return Promise.resolve(["binance", "kraken"]);
                return Promise.resolve(JSON.stringify(["binance", "kraken"]));
              }
              return Promise.resolve(null);
            }
          ),
        put: vi.fn().mockResolvedValue(undefined),
        delete: vi.fn().mockResolvedValue(undefined),
        list: vi.fn().mockResolvedValue({
          keys: [],
          list_complete: true,
          cursor: undefined,
        }),
        getWithMetadata: vi
          .fn()
          .mockResolvedValue({ value: null, metadata: null }),
      } as unknown as KVNamespace,
    };

    const mocks = testAccessibleMockInstances;
    if (mocks) {
      for (const exchangeIdKey in mocks) {
        if (Object.prototype.hasOwnProperty.call(mocks, exchangeIdKey)) {
          const exchangeId = exchangeIdKey as ExchangeId;
          const instance = mocks[exchangeId];
          for (const mockFn of Object.values(instance)) {
            if (
              typeof mockFn === "function" &&
              "_isMockFunction" in mockFn &&
              mockFn._isMockFunction
            ) {
              (mockFn as Mock).mockClear();
            }
          }
          instance.loadMarkets.mockResolvedValue({
            "BTC/USDT": MOCK_MARKET_DEFAULTS as Market,
          });
          instance.fetchBalance.mockResolvedValue(
            MOCK_BINANCE_BALANCES_FACTORY()
          );
        }
      }
    }
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should add exchanges and load their markets when added", async () => {
    const service = new ExchangeService({ logger: logger, env: mockEnv });

    // Get/initialize exchanges explicitly, which also loads their markets
    await service.getExchangeInstance("binance");
    await service.getExchangeInstance("kraken");

    const mocks = testAccessibleMockInstances;
    if (!mocks?.binance || !mocks?.kraken) {
      throw new Error(
        "Mocks for binance or kraken not initialized after adding them."
      );
    }

    // addExchange should trigger loadMarkets for each respective exchange instance.
    expect(mocks.binance.loadMarkets).toHaveBeenCalledTimes(1);
    expect(mocks.kraken.loadMarkets).toHaveBeenCalledTimes(1);
  });

  describe("Dynamic Exchange Management", () => {
    let service: ExchangeService;

    beforeEach(() => {
      service = new ExchangeService({ logger: logger, env: mockEnv });
      // Resetting mocks for specific exchanges if they were manipulated in other tests
      // Ensure that testAccessibleMockInstances are fresh or reset for kraken
      if (testAccessibleMockInstances?.kraken?.loadMarkets) {
        (testAccessibleMockInstances.kraken.loadMarkets as Mock).mockClear();
      }
    });

    it("should initialize an exchange and its markets upon first retrieval", async () => {
      // kraken mock should exist from the global ccxt mock setup
      const mocks = testAccessibleMockInstances;
      if (!mocks?.kraken)
        throw new Error("Kraken mock not available in test setup");

      // Ensure loadMarkets hasn't been called yet for kraken in this test context
      // (Note: if kraken was a DEFAULT_EXCHANGE and initialized in constructor, this would be different)
      // For this test, we assume kraken is NOT a default exchange pre-initialized.
      expect(mocks.kraken.loadMarkets).not.toHaveBeenCalled();

      const krakenInstance = await service.getExchangeInstance("kraken");

      expect(krakenInstance).toBeDefined();
      expect(krakenInstance?.id).toBe("kraken");
      expect(mocks.kraken.loadMarkets).toHaveBeenCalledTimes(1);
    });

    /* Test for a non-existent feature removeExchange
    it('should remove an exchange', async () => {
      // Re-initialize service for this specific test case to start with both exchanges.
      service = new ExchangeService({ logger: logger, env: mockEnv });
      expect(service.getExchangeInstance('binance')).toBeDefined();
      expect(service.getExchangeInstance('kraken')).toBeDefined();

      service.removeExchange('kraken');
      expect(service.getExchangeInstance('kraken')).toBeUndefined();
      // Optionally, check that the binance instance still exists
      expect(service.getExchangeInstance('binance')).toBeDefined();
    });
    */
  });

  describe("getOpenPositions", () => {
    let service: ExchangeService;

    beforeEach(() => {
      // Ensure mocks are clean or specifically set for these tests if necessary
      // The main beforeEach already clears mocks, so this might be fine.
      service = new ExchangeService({ logger: logger, env: mockEnv });
      // No explicit await service.initializeMarkets() is needed here because
      // the ExchangeService constructor handles loading markets for 'binance'.
    });

    it("should fetch open positions for a given symbol on an exchange", async () => {
      const mocks = testAccessibleMockInstances!;
      const fakePosition = {
        symbol: "BTC/USDT",
        id: "1",
        timestamp: Date.now(),
        datetime: new Date().toISOString(),
        info: {},
        type: undefined,
        side: "long",
        contracts: 1,
        contractSize: undefined,
        price: 20000,
        entryPrice: 20000,
        markPrice: 20000,
        notional: 20000,
        leverage: 1,
        collateral: 20000,
        initialMargin: 20000,
        maintenanceMargin: 1000,
        unrealizedPnl: 0,
        realizedPnl: 0,
        liquidationPrice: 10000,
        marginMode: "isolated",
        hedged: false,
        maintenanceMarginPercentage: undefined,
        initialMarginPercentage: undefined,
        percentage: undefined,
      };
      mocks.binance.fetchPositions.mockResolvedValue([fakePosition] as any);

      const positions = await service.getOpenPositions("binance", "BTC/USDT");
      expect(positions).toBeDefined();
      expect(positions).not.toBeNull();
      expect(positions!.length).toBeGreaterThan(0);
      expect(positions?.[0]?.symbol).toBe("BTC/USDT");
      expect(mocks.binance.fetchPositions).toHaveBeenCalledTimes(1);
    });
  });

  describe("setLeverage", () => {
    let service: ExchangeService;

    beforeEach(() => {
      service = new ExchangeService({ logger: logger, env: mockEnv });
    });

    it("should set leverage for a symbol on a given exchange", async () => {
      const mocks = testAccessibleMockInstances!;
      const fakeLeverageResponse = { info: "Leverage set to 10 for BTC/USDT" };
      mocks.binance.setLeverage.mockResolvedValue(fakeLeverageResponse as any);

      const leverageResult = await service.setLeverage(
        "binance",
        "BTC/USDT",
        10
      );
      expect(leverageResult).toBeDefined();
      expect(leverageResult).toEqual(fakeLeverageResponse);
      expect(mocks.binance.setLeverage).toHaveBeenCalledWith(
        10,
        "BTC/USDT",
        undefined
      ); // Check specific call args if params is not used
    });
  });

  describe("getTicker", () => {
    let service: ExchangeService;

    beforeEach(() => {
      service = new ExchangeService({ logger: logger, env: mockEnv });
    });

    it("should fetch the ticker for a symbol on a given exchange", async () => {
      const mocks = testAccessibleMockInstances!;
      // if (!mocks) throw new Error("Mocks not initialized for test 'getTicker'"); // Covered by non-null assertion

      // Explicitly mock fetchTicker for this test
      const fakeTicker = {
        symbol: "BTC/USDT",
        bid: 100,
        ask: 101,
        timestamp: Date.now(),
        datetime: new Date().toISOString(),
        last: 100.5,
        info: {},
        // Add other required Ticker fields if necessary, e.g., from ccxt.Ticker type if strict
        high: undefined,
        low: undefined,
        vwap: undefined,
        open: undefined,
        close: undefined,
        average: undefined,
        baseVolume: undefined,
        quoteVolume: undefined,
        previousClose: undefined,
        change: undefined,
        percentage: undefined,
      };
      mocks.binance.fetchTicker.mockResolvedValue(fakeTicker as any); // Use 'as any' to bypass strict Ticker typing for the mock value if needed

      const ticker = await service.getTicker("binance", "BTC/USDT");

      expect(mocks.binance.fetchTicker).toHaveBeenCalledWith("BTC/USDT");
      expect(ticker).toBeDefined();
      expect(ticker?.symbol).toBe("BTC/USDT");
      expect(ticker?.last).toBe(100.5); // Check a value from the explicit mock
    });
  });

  describe("Error Handling", () => {
    let service: ExchangeService;

    beforeEach(() => {
      service = new ExchangeService({ logger: logger, env: mockEnv });
    });

    it("should return null when an exchange method throws an error", async () => {
      const mocks = testAccessibleMockInstances;
      if (!mocks)
        throw new Error("Mocks not initialized for test 'Error Handling'");

      // Ensure the specific mock for binance is available and then configure its behavior
      if (!mocks.binance)
        throw new Error("Binance mock not available for 'Error Handling' test");

      mocks.binance.has = {
        ...(mocks.binance.has || {}),
        fetchTicker: true,
      };
      (mocks.binance.fetchTicker as Mock).mockRejectedValue(
        new Error("API Error")
      );

      const ticker = await service.getTicker("binance", "BTC/USDT");
      expect(ticker).toBeNull();
      expect(mocks.binance.fetchTicker).toHaveBeenCalledWith("BTC/USDT");
    });
  });

  afterAll(() => {
    vi.doUnmock("ccxt");
    vi.resetModules();
  });
});
