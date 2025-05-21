// Webhook Endpoint Tests
import type {
  ExecutionContext,
  KVNamespace, // For TestEnv
  DurableObjectNamespace, // For TestEnv
} from "@cloudflare/workers-types";
import type { Bot } from "grammy";
import type { Update } from "grammy/types"; // Corrected import for Update
import { webhookCallback } from "grammy";
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
import httpWorker from "../src/index";

// Service imports
import { TelegramService } from "../src/services/telegramService";

// Mock helpers
import { type MockLogger, createMockLogger } from "./__mocks__/logger";
import {
  createSimpleMockKvNamespace,
  createMockDurableObjectNamespace,
} from "./utils/kv.mock";

// Type imports from src
import type {
  Env as ImportedEnv,
  // Other types if used by TestEnv or specific tests
} from "../src/types";

// --- BEGIN MOCK DEFINITIONS (copied/adapted from index.test.ts) ---
const mockLoggerInstanceGlobal = createMockLogger(); // Renamed to avoid conflict if mockLoggerInstance is used locally in tests
vi.mock("../src/utils/logger", () => ({
  createLogger: vi.fn(() => mockLoggerInstanceGlobal),
}));

const mockBotInstance = {
  on: vi.fn(),
  start: vi.fn(),
  stop: vi.fn(),
  handleUpdate: vi.fn(),
  api: {
    sendMessage: vi.fn().mockResolvedValue({ ok: true, result: {} }),
    sendMediaGroup: vi.fn().mockResolvedValue({ ok: true, result: [] }),
    setMyCommands: vi.fn().mockResolvedValue(true),
    deleteWebhook: vi.fn().mockResolvedValue(true),
    setWebhook: vi.fn().mockResolvedValue(true),
  },
  botInfo: { id: 123456, is_bot: true, username: "TestBot" },
};

const mockTelegramServiceInstance = {
  sendMessage: vi.fn().mockResolvedValue(undefined),
  sendMessages: vi.fn().mockResolvedValue(undefined),
  sendOpportunityNotification: vi.fn().mockResolvedValue(undefined),
  processWebhook: vi.fn().mockResolvedValue(undefined),
  findArbitrageOpportunities: vi.fn().mockResolvedValue([]),
  processFundingRateOpportunities: vi.fn().mockResolvedValue(undefined),
  stop: vi.fn(),
  logger: mockLoggerInstanceGlobal, 
  getBotInstance: vi.fn(() => mockBotInstance),
} as unknown as TelegramService;

vi.mock("../src/services/telegramService", () => ({
  TelegramService: vi.fn(() => mockTelegramServiceInstance),
}));

// Grammy mock is crucial for webhook tests
vi.mock("grammy", async () => {
  const actualGrammy = await vi.importActual<typeof import("grammy")>("grammy");
  const mockWebhookCb = vi.fn(
    (botPassedToCallback: Bot, framework: string) => {
      return async (req: Request) => {
        // Simplified mock for tests; specific tests might override parts of this
        if (botPassedToCallback && typeof botPassedToCallback.handleUpdate === "function") {
          try {
            const update = await req.json() as Update;
            await botPassedToCallback.handleUpdate(update);
          } catch (e) { 
            // console.error("Error in default mockWebhookCb internal try-catch:", e); // Optional debug log
            throw e; // Rethrow error
          }
        }
        return new Response("mock webhook processed", { status: 200 });
      };
    }
  );
  const MockedBotConstructor = vi.fn().mockImplementation((token, config) => mockBotInstance);
  return {
    ...actualGrammy,
    Bot: MockedBotConstructor,
    webhookCallback: mockWebhookCb, // Use the local mockWebhookCb
  };
});
// --- END MOCK DEFINITIONS ---

// --- Test Environment Setup ---
type TestEnv = {
  LOG_LEVEL: string;
  TELEGRAM_BOT_TOKEN: string;
  TELEGRAM_CHAT_ID: string;
  EXCHANGES: string; // Needed for base TestEnv, even if not directly used
  MONITORED_PAIRS_CONFIG: string; // Needed for base TestEnv
  ARBITRAGE_THRESHOLD: string; // Needed for base TestEnv
  LOGGER: MockLogger;
  ArbEdgeKV: KVNamespace;
  POSITIONS: DurableObjectNamespace;
  telegramServiceInstance?: TelegramService;
  // Add other service instances if they become part of TestEnv and are needed
} & Omit<ImportedEnv, "LOGGER" | "ArbEdgeKV" | "POSITIONS" | "EXCHANGES" | "MONITORED_PAIRS_CONFIG" | "ARBITRAGE_THRESHOLD">;

const mockRequest = (method: string, url: string, body?: unknown): Request => {
  const options: RequestInit = { method };
  if (body !== undefined) {
    options.body = JSON.stringify(body);
    options.headers = { "Content-Type": "application/json" };
  }
  return new Request(url, options);
};

let mockEnv: TestEnv;
let mockExecutionContext: ExecutionContext;
let localMockLogger: MockLogger; // For tests that initialize their own logger instance

// --- END Test Environment Setup ---

// Original test file: tests/index.test.ts
describe("Worker Webhook Logic", () => {

  beforeEach(() => {
    // Reset global mocks and instances for each test
    vi.clearAllMocks();
    localMockLogger = createMockLogger(); // Use this for most direct LOGGER assignments

    // Setup mockEnv for each test
    mockEnv = {
      LOG_LEVEL: "debug",
      TELEGRAM_BOT_TOKEN: "test_bot_token",
      TELEGRAM_CHAT_ID: "test_chat_id",
      EXCHANGES: "mockex",
      MONITORED_PAIRS_CONFIG: JSON.stringify([{ symbol: "BTC/USDT" }]),
      ARBITRAGE_THRESHOLD: "0.1",
      LOGGER: localMockLogger,
      ArbEdgeKV: createSimpleMockKvNamespace(),
      POSITIONS: createMockDurableObjectNamespace(),
      telegramServiceInstance: mockTelegramServiceInstance, // Pre-inject the global mock instance
    } as TestEnv;

    mockExecutionContext = {
      waitUntil: vi.fn(),
      passThroughOnException: vi.fn(),
      props: {},
    } as ExecutionContext;

    // Reset the webhookCallback mock from grammy to a default behavior for each test
    // This allows individual tests to specialize it if needed.
    const defaultWebhookHandler = vi.fn().mockResolvedValue(new Response("OK from default webhook mock", { status: 200 }));
    vi.mocked(webhookCallback).mockReturnValue(defaultWebhookHandler);
  });

  afterEach(() => {
    vi.restoreAllMocks(); // Restore any spied/altered mocks
  });

  // Copied from tests/index.test.ts (around line 464)
  it("should handle invalid JSON in webhook", async () => {
    const request = new Request("http://localhost/webhook", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "invalid json string { not json }",
    });
    const mockJsonParseError = new Error("Simulated JSON parse error");
    vi.mocked(webhookCallback).mockImplementationOnce(() => 
      vi.fn().mockRejectedValue(mockJsonParseError)
    );
    const response = await httpWorker.fetch(request, mockEnv, mockExecutionContext);
    expect(response.status).toBe(500);
    expect(await response.text()).toContain("Internal Server Error: Simulated JSON parse error");
    // Check logger if error is logged through the global mock
    // expect(mockLoggerInstanceGlobal.error).toHaveBeenCalledWith(expect.stringContaining("JSON"), expect.any(Error));
  });

  // Copied from tests/index.test.ts (around line 490)
  it("should return 500 for POST /webhook with non-JSON content type", async () => {
    const request = new Request("http://localhost/webhook", {
      method: "POST",
      headers: { "Content-Type": "text/plain" }, // Non-JSON
      body: "Test webhook body",
    });
    const mockContentTypeError = new Error("Simulated content type error");
    vi.mocked(webhookCallback).mockImplementationOnce(() => 
      vi.fn().mockRejectedValue(mockContentTypeError)
    );
    const response = await httpWorker.fetch(request, mockEnv, mockExecutionContext);
    expect(response.status).toBe(500);
    expect(await response.text()).toContain("Internal Server Error: Simulated content type error");
  });

  // Copied describe block from tests/index.test.ts (around line 755)
  describe("/webhook endpoint specific tests", () => {
    // beforeEach for this inner describe is inherited or can be added if specific setup needed

    it("should return 500 if TELEGRAM_BOT_TOKEN is missing", async () => {
      const testEnvWithoutToken = {
        ...mockEnv,
        TELEGRAM_BOT_TOKEN: "", // Intentionally missing
        telegramServiceInstance: undefined, // Ensure new service would be created
        LOGGER: localMockLogger, // Use the locally scoped logger for this test
      } as TestEnv;
      const request = mockRequest("POST", "http://localhost/webhook", {});
      const response = await httpWorker.fetch(request, testEnvWithoutToken, mockExecutionContext);
      expect(response.status).toBe(500);
      expect(await response.text()).toBe("Telegram secrets not configured");
      expect(localMockLogger.error).toHaveBeenCalledWith("Telegram secrets not configured in environment.");
    });

    it("should return 500 if TELEGRAM_CHAT_ID is missing", async () => {
      const testEnvWithoutChatId = {
        ...mockEnv,
        TELEGRAM_CHAT_ID: "", // Intentionally missing
        telegramServiceInstance: undefined, // Ensure new service would be created
        LOGGER: localMockLogger,
      } as TestEnv;
      const request = mockRequest("POST", "http://localhost/webhook", {});
      const response = await httpWorker.fetch(request, testEnvWithoutChatId, mockExecutionContext);
      expect(response.status).toBe(500);
      expect(await response.text()).toBe("Telegram secrets not configured");
      expect(localMockLogger.error).toHaveBeenCalledWith("Telegram secrets not configured in environment.");
    });

    it("should correctly initialize TelegramService and call webhookCallback if service NOT on env", async () => {
      const currentMockEnv: TestEnv = {
        ...mockEnv,
        telegramServiceInstance: undefined, // Simulate service not being on env
        LOGGER: localMockLogger,
      };
      const request = mockRequest("POST", "http://localhost/webhook", { update_id: 1 });
      
      // This test relies on the global vi.mock("grammy") for webhookCallback behavior
      // and vi.mock for TelegramService constructor behavior

      await httpWorker.fetch(request, currentMockEnv, mockExecutionContext);

      expect(TelegramService).toHaveBeenCalledWith(
        expect.objectContaining({
          botToken: currentMockEnv.TELEGRAM_BOT_TOKEN,
          chatId: currentMockEnv.TELEGRAM_CHAT_ID,
          logger: localMockLogger, 
        }),
        expect.objectContaining({ env: "production_webhook", startPolling: false })
      );
      // The actual call to webhookCallback happens inside the worker's route handler
      // We check if it was called by the mock setup
      expect(webhookCallback).toHaveBeenCalled(); 
    });

    it("should use existing TelegramService instance from env and call webhookCallback", async () => {
      // mockEnv in the outer beforeEach already has telegramServiceInstance (mockTelegramServiceInstance)
      const request = mockRequest("POST", "http://localhost/webhook", { update_id: 123 });

      await httpWorker.fetch(request, mockEnv, mockExecutionContext);

      expect(TelegramService).not.toHaveBeenCalled(); // Constructor should NOT be called
      expect(webhookCallback).toHaveBeenCalledTimes(1);
      expect(webhookCallback).toHaveBeenCalledWith(
        mockTelegramServiceInstance.getBotInstance(), // Expecting the bot from the instance on env
        "cloudflare-mod"
      );
      // Check if the handler returned by webhookCallback was called
      const mockReturnedHandler = vi.mocked(webhookCallback).mock.results[0].value;
      expect(mockReturnedHandler).toHaveBeenCalledWith(request);
    });

    it("should return 500 and log if webhookCallback throws an Error", async () => {
      const request = mockRequest("POST", "http://localhost/webhook", { update_id: 1 });
      const webhookError = new Error("Webhook processing failed deliberately");

      // Make webhookCallback return a function that, when called, returns a promise that rejects.
      vi.mocked(webhookCallback).mockImplementationOnce(() => {
        return vi.fn().mockRejectedValue(webhookError); // This is handleUpdate, returns a rejecting promise
      });

      const response = await httpWorker.fetch(request, mockEnv, mockExecutionContext);

      expect(response.status).toBe(500);
      expect(await response.text()).toContain("Internal Server Error: Webhook processing failed deliberately");
      expect(mockEnv.LOGGER.error).toHaveBeenCalledWith(
        "Webhook processing error (Error instance):",
        "Webhook processing failed deliberately",
        webhookError.stack
      );
    });

    it("should return 500 and log if webhookCallback throws a non-Error", async () => {
      const request = mockRequest("POST", "http://localhost/webhook", {});
      const nonErrorObject = { message: "Something went wrong deliberately as non-error" };
      vi.mocked(webhookCallback).mockReturnValue(vi.fn().mockRejectedValue(nonErrorObject));
      
      const response = await httpWorker.fetch(request, mockEnv, mockExecutionContext);
      
      expect(response.status).toBe(500);
      expect(await response.text()).toBe("Internal Server Error: An unexpected error occurred");
      expect(localMockLogger.error).toHaveBeenCalledWith("Webhook caught error object:", nonErrorObject);
      expect(localMockLogger.error).toHaveBeenCalledWith("Webhook processing error (unknown type):", nonErrorObject);
    });

    // This is the test that was previously failing due to mockEnv scope
    it("should return 500 if Telegram secrets are not configured (direct test of problematic scenario)", async () => {
      const localRequest = new Request("http://localhost/webhook", {
        method: "POST",
        body: JSON.stringify({ update_id: 123 }),
        headers: { "Content-Type": "application/json" },
      });
      const currentTestEnv = {
        ...mockEnv, // Base mockEnv
        TELEGRAM_BOT_TOKEN: undefined, // Undefine token
        TELEGRAM_CHAT_ID: undefined,   // Undefine chat ID
        telegramServiceInstance: undefined, // Ensure service would be re-created
        LOGGER: localMockLogger,
      } as unknown as TestEnv;
      
      const localMockCtx: ExecutionContext = {
        waitUntil: vi.fn(),
        passThroughOnException: vi.fn(),
        props: {},
      };

      const response = await httpWorker.fetch(localRequest, currentTestEnv, localMockCtx);

      expect(response.status).toBe(500);
      expect(await response.text()).toBe("Telegram secrets not configured");
      expect(localMockLogger.error).toHaveBeenCalledWith("Telegram secrets not configured in environment.");
    });
  });
}); 