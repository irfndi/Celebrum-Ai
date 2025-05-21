//гіїScheduled Worker Tests
import type {
  ExecutionContext,
  ScheduledController,
  KVNamespace, // For TestEnv
  DurableObjectNamespace, // For TestEnv
} from "@cloudflare/workers-types";
import {
  vi,
  describe,
  it,
  expect,
  beforeEach,
  afterEach,
  type Mock,
} from "vitest";

// Worker import
import httpWorker from "../src/index"; // Adjust path if worker is in src/
// Service imports
import { ExchangeService } from "../src/services/exchangeService"; // Restored non-type import
import { OpportunityService } from "../src/services/opportunityService"; // Restored non-type import
import type { TelegramService } from "../src/services/telegramService"; // Added non-type import for TelegramService
import type { OpportunityServiceConfig } from "../src/services/opportunityService"; // Corrected import location
import type { ArbitrageOpportunity } from "../src/types"; // Import ArbitrageOpportunity

// Mock helpers
import { type MockLogger, createMockLogger } from "./__mocks__/logger";
import {
  createSimpleMockKvNamespace,
  createMockDurableObjectNamespace,
} from "./utils/kv.mock";

// Type imports from src (assuming they are in ../../src/types)
import type {
  Env as ImportedEnv, // For TestEnv
  StructuredTradingPair, // For scheduled test
} from "../src/types"; // Corrected path

// --- BEGIN MOCK DEFINITIONS (copied from index.test.ts, simplified if not all needed) ---
const mockExchangeServiceInstanceGlobal = {
  exchanges: {},
  getExchangeIds: vi.fn().mockReturnValue(["binance", "bybit"]),
  addExchange: vi.fn(),
  getTicker: vi.fn(),
  getOrderBook: vi.fn(),
} as unknown as ExchangeService;

vi.mock("../src/services/exchangeService", () => {
  const MockExchangeServiceConstructor = vi
    .fn()
    .mockImplementation(() => mockExchangeServiceInstanceGlobal);
  return { ExchangeService: MockExchangeServiceConstructor };
});

const mockTelegramServiceInstance = {
  logger: createMockLogger(),
  getBotInstance: vi.fn(() => ({ api: { sendMessage: vi.fn() } })),
  sendOpportunityNotification: vi.fn().mockResolvedValue(undefined),
} as unknown as TelegramService;

vi.mock("../src/services/telegramService", () => ({
  TelegramService: vi.fn(() => mockTelegramServiceInstance),
}));

// Define the specialized telegram mock as a constant, DO NOT RE-MOCK THE MODULE
const mockTelegramServiceForScheduledTestConfig = {
  sendOpportunityNotification: vi.fn().mockResolvedValue(undefined),
  logger: createMockLogger(), 
  getBotInstance: vi.fn(), 
} as unknown as TelegramService;

const mockOpportunityServiceActualInstance = {
  findOpportunities: vi.fn().mockResolvedValue([]),
  processFundingRateOpportunities: vi.fn().mockResolvedValue(undefined),
  logger: createMockLogger(),
  getConfig: vi.fn().mockImplementation(
    () =>
      ({
        exchangeService: mockExchangeServiceInstanceGlobal,
        telegramService: mockTelegramServiceInstance, 
        logger: createMockLogger(),
        monitoredPairs: [],
        exchanges: [],
        threshold: 0.001,
      }) as OpportunityServiceConfig
  ),
} as unknown as OpportunityService;

vi.mock("../src/services/opportunityService", () => ({
  OpportunityService: vi.fn(() => mockOpportunityServiceActualInstance),
}));

const actualMockLoggerInstance = createMockLogger();
vi.mock("../src/utils/logger", () => ({
  createLogger: vi.fn(() => actualMockLoggerInstance),
}));
// --- END MOCK DEFINITIONS ---

// --- Test Environment Setup (copied and adapted from index.test.ts) ---
interface MockScheduledController
  extends ScheduledController,
    Pick<ExecutionContext, "waitUntil"> {
  noRetry: Mock<() => void>;
  waitUntil: Mock<(p: Promise<unknown>) => void>;
}

const createMockScheduledController = (): MockScheduledController => ({
  scheduledTime: Date.now(),
  cron: "0 * * * *",
  noRetry: vi.fn(),
  waitUntil: vi.fn(),
});

// Define TestEnv if not imported from a shared location
type TestEnv = {
  LOG_LEVEL: string;
  TELEGRAM_BOT_TOKEN: string;
  TELEGRAM_CHAT_ID: string;
  EXCHANGES: string;
  MONITORED_PAIRS_CONFIG: string;
  ARBITRAGE_THRESHOLD: string;
  LOGGER: MockLogger;
  ArbEdgeKV: KVNamespace;
  POSITIONS: DurableObjectNamespace;
  telegramServiceInstance?: TelegramService;
  opportunityServiceInstance?: OpportunityService;
  exchangeServiceInstance?: ExchangeService;
} & Omit<ImportedEnv, "LOGGER" | "ArbEdgeKV" | "POSITIONS">;

// Cloudflare test helpers (consider if these are needed directly or if a simpler mockCtx is enough)
// For now, just a basic mockCtx for the scheduled tests
const cfCreateExecutionContext = (): ExecutionContext => ({
  waitUntil: vi.fn(),
  passThroughOnException: vi.fn(),
  props: {}, // Added props
});

// --- END Test Environment Setup ---

// Copied describe block from tests/index.test.ts (around line 946)
// Original test file: tests/index.test.ts
describe("Worker Scheduled Handler Logic", () => {
  // Tests for the scheduled handler
  describe("scheduled handler direct tests", () => {
    let mockController: MockScheduledController;
    let mockCtxScheduled: ExecutionContext;
    let baseMockEnvScheduled: TestEnv;
    let localMockLoggerScheduled: MockLogger;

    beforeEach(() => {
      vi.clearAllMocks();
      localMockLoggerScheduled = createMockLogger();
      mockController = createMockScheduledController();
      mockCtxScheduled = { waitUntil: vi.fn(), passThroughOnException: vi.fn(), props: {} }; // Added props


      baseMockEnvScheduled = {
        LOG_LEVEL: "debug",
        TELEGRAM_BOT_TOKEN: "test-token-scheduled-base",
        TELEGRAM_CHAT_ID: "chat-id-scheduled-base",
        EXCHANGES: "scheduled-base-ex1,scheduled-base-ex2",
        MONITORED_PAIRS_CONFIG: JSON.stringify([
          { symbol: "BASE/USDT", base: "BASE", quote: "USDT", type: "spot" },
        ]),
        ARBITRAGE_THRESHOLD: "0.0099",
        LOGGER: localMockLoggerScheduled,
        ArbEdgeKV: createSimpleMockKvNamespace(),
        POSITIONS: createMockDurableObjectNamespace(),
      } as TestEnv; // Cast to ensure all properties are covered if some are optional in base ImportedEnv
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("scheduled handler should create new ExchangeService and OpportunityService if not injected", async () => {
      const envForNewServices: TestEnv = {
        ...baseMockEnvScheduled,
        TELEGRAM_BOT_TOKEN: undefined as unknown as string, // Ensure TelegramService is not created
        TELEGRAM_CHAT_ID: undefined as unknown as string,   // Ensure TelegramService is not created
        EXCHANGES: "dynamicEx1,dynamicEx2",
        MONITORED_PAIRS_CONFIG: JSON.stringify([
          { symbol: "DYN/USDT", base: "DYN", quote: "USDT", type: "swap" },
        ]),
        ARBITRAGE_THRESHOLD: "0.0025",
        telegramServiceInstance: undefined,
        exchangeServiceInstance: undefined,
        opportunityServiceInstance: undefined,
      };

      const expectedMonitoredPairs = JSON.parse(
        envForNewServices.MONITORED_PAIRS_CONFIG
      );
      const expectedExchanges = envForNewServices.EXCHANGES.split(",").map(
        (ex) => ex.trim()
      );
      const expectedThreshold = Number.parseFloat(
        envForNewServices.ARBITRAGE_THRESHOLD
      );

      (mockOpportunityServiceActualInstance.getConfig as Mock).mockReturnValue({
        exchangeService: mockExchangeServiceInstanceGlobal,
        telegramService: null, 
        logger: localMockLoggerScheduled,
        monitoredPairs: expectedMonitoredPairs,
        exchanges: expectedExchanges,
        threshold: expectedThreshold,
      });

      const scheduledHandler = httpWorker.scheduled;
      if (!scheduledHandler) {
        throw new Error("Scheduled handler is not defined on the worker.");
      }

      await scheduledHandler(
        mockController,
        envForNewServices,
        mockCtxScheduled
      );

      expect(vi.mocked(ExchangeService)).toHaveBeenCalledWith({
        env: expect.objectContaining({ EXCHANGES: "dynamicEx1,dynamicEx2" }),
        logger: localMockLoggerScheduled,
      });

      expect(vi.mocked(OpportunityService)).toHaveBeenCalledWith({
        exchangeService: mockExchangeServiceInstanceGlobal,
        telegramService: null, 
        logger: localMockLoggerScheduled,
        monitoredPairs: expectedMonitoredPairs,
        exchanges: expectedExchanges,
        threshold: expectedThreshold,
      });

      expect(
        mockOpportunityServiceActualInstance.findOpportunities as Mock
      ).toHaveBeenCalledTimes(1);
      expect(
        mockOpportunityServiceActualInstance.findOpportunities as Mock
      ).toHaveBeenCalledWith(
        expectedExchanges,
        expectedMonitoredPairs.map((p: StructuredTradingPair) => p.symbol),
        expectedThreshold
      );
    });
  });

  // Copied describe block from tests/index.test.ts (around line 1026)
  describe("additional coverage for scheduled/cron paths", () => {
    let mockLogger: MockLogger;
    let mockController: MockScheduledController;
    let mockCtx: ExecutionContext;
    // let worker: typeof import("../../src/index"); // Use httpWorker directly

    beforeEach(async () => {
      vi.clearAllMocks(); // Ensure mocks are cleared for this describe block too
      mockLogger = createMockLogger();
      mockController = createMockScheduledController();
      mockCtx = { // Simplified mock context
        waitUntil: vi.fn(),
        passThroughOnException: vi.fn(),
        props: {} // Added props
      } as unknown as ExecutionContext;
      // Worker is imported at the top as httpWorker
      // If fresh import per test is needed, uncomment and adapt:
      // worker = await import("../../src/index"); 
    });

    afterEach(() => {
      vi.clearAllMocks();
      // vi.resetModules(); // Only if using dynamic import per test for worker
    });

    it("should handle cron job with service instances already in env", async () => {
      const mockLocalOpportunityService = {
        getConfig: vi.fn().mockReturnValue({
          exchanges: ["binance", "bybit"],
          monitoredPairs: [{ symbol: "BTC/USDT", base: "BTC", quote: "USDT", type: "swap" } as StructuredTradingPair],
          threshold: 0.002,
          telegramService: mockTelegramServiceForScheduledTestConfig, // Corrected: Use the renamed constant
        }),
        findOpportunities: vi.fn().mockResolvedValue([]), // MODIFIED: Return empty array
      };

      const mockEnvWithInstance: TestEnv = {
        opportunityServiceInstance: mockLocalOpportunityService as unknown as OpportunityService,
        LOGGER: mockLogger,
        EXCHANGES: "binance,bybit", // Required by TestEnv, though might be ignored by logic if instance is present
        ARBITRAGE_THRESHOLD: "0.001", // Required by TestEnv
        MONITORED_PAIRS_CONFIG: JSON.stringify([
          { symbol: "BTC/USDT" },
          { symbol: "ETH/USDT" },
        ]), // Required by TestEnv
        // Fill other required TestEnv properties for completeness
        LOG_LEVEL: "info",
        TELEGRAM_BOT_TOKEN: "token",
        TELEGRAM_CHAT_ID: "chatid",
        ArbEdgeKV: createSimpleMockKvNamespace(),
        POSITIONS: createMockDurableObjectNamespace(),
      } as TestEnv;

      const scheduledHandler = httpWorker.scheduled;
      if (!scheduledHandler) throw new Error("Scheduled handler undefined");

      await scheduledHandler(
        mockController as unknown as ScheduledController,
        mockEnvWithInstance,
        mockCtx
      );

      expect(mockLocalOpportunityService.getConfig).toHaveBeenCalled();
      expect(mockLocalOpportunityService.findOpportunities).toHaveBeenCalledWith(
        ["binance", "bybit"], // Expect specific exchanges from getConfig
        ["BTC/USDT"],        // Expect specific symbols from getConfig
        0.002                // Expect specific threshold from getConfig
      );
      expect(mockLogger.info).toHaveBeenCalledWith(
        "Scheduled task processed successfully."
      );
    });

    it("should handle JSON parse error in cron job", async () => {
      const mockEnvJsonError: TestEnv = {
        LOGGER: mockLogger,
        EXCHANGES: "binance,bybit",
        ARBITRAGE_THRESHOLD: "0.001",
        MONITORED_PAIRS_CONFIG: "invalid-json", // This will cause a JSON parse error
        // Fill other required TestEnv properties
        LOG_LEVEL: "info",
        TELEGRAM_BOT_TOKEN: "token",
        TELEGRAM_CHAT_ID: "chatid",
        ArbEdgeKV: createSimpleMockKvNamespace(),
        POSITIONS: createMockDurableObjectNamespace(),
      } as TestEnv;

      const scheduledHandler = httpWorker.scheduled;
      if (!scheduledHandler) throw new Error("Scheduled handler undefined");

      await scheduledHandler(
        mockController as unknown as ScheduledController,
        mockEnvJsonError,
        mockCtx
      );

      expect(mockLogger.error).toHaveBeenCalledWith(
        "Failed to parse MONITORED_PAIRS_CONFIG from env for scheduled task:",
        expect.any(String),
        expect.any(Error)
      );
    });

    it("should handle error in findOpportunities during scheduled execution", async () => {
      // Mock the opportunityService to throw an error
      const testError = new Error("Network error in findOpportunities");
      const mockOpportunityService = {
        findOpportunities: vi.fn().mockRejectedValue(testError),
        getConfig: vi.fn().mockReturnValue({
          exchanges: ["binance", "bybit"],
          monitoredPairs: [{ symbol: "BTC/USDT" }],
          threshold: 0.001,
        }),
      };
      
      // Create environment with injected opportunity service
      const mockEnv = {
        LOGGER: mockLogger,
        EXCHANGES: "binance,bybit",
        MONITORED_PAIRS_CONFIG: JSON.stringify([{ symbol: "BTC/USDT" }]),
        ARBITRAGE_THRESHOLD: "0.001",
        LOG_LEVEL: "info",
        TELEGRAM_BOT_TOKEN: "token",
        TELEGRAM_CHAT_ID: "chatid",
        ArbEdgeKV: createSimpleMockKvNamespace(),
        POSITIONS: createMockDurableObjectNamespace(),
        opportunityServiceInstance: mockOpportunityService as unknown as OpportunityService,
      } as TestEnv;
      
      // Use the worker's scheduled method directly
      await httpWorker.scheduled(
        mockController as unknown as ScheduledController,
        mockEnv,
        mockCtx
      );
      
      expect(mockLogger.error).toHaveBeenCalledWith(
        "Error in scheduled handler execution:",
        "Network error in findOpportunities",
        testError
      );
    });
  });
}); 