/// <reference types="vitest/globals" />
import {
  describe,
  expect,
  it,
  vi,
  beforeEach,
  afterEach,
  test,
  type MockInstance,
  type MockedFunction,
} from "vitest";
import { TelegramService } from "../../src/services/telegramService";
import type { ArbitrageOpportunity, LoggerInterface } from "../../src/types";
import { Bot, GrammyError, HttpError } from "grammy";
import type { Context } from "grammy";
import type { Update, Message, ApiError } from "grammy/types"; // ApiError still assumed here

declare global {
  // eslint-disable-next-line no-var
  var testStoredErrorHandler:
    | ((err: {
        error: Error;
        ctx: Partial<
          Context & {
            update: Update;
            reply: MockInstance<(text: string) => Promise<Message.TextMessage>>;
          }
        >;
      }) => void)
    | undefined;
}
import * as formatterUtils from "../../src/utils/formatter";

// Mock grammy
vi.mock("grammy", () => {
  return {
    Bot: vi.fn().mockImplementation(() => ({
      api: {
        sendMessage: vi.fn().mockResolvedValue({
          message_id: 1,
          date: Math.floor(Date.now() / 1000),
          chat: { id: 12345, type: "private", first_name: "Test Bot User" },
          text: "Mocked send message",
        } as Message.TextMessage),
      },
      catch: vi.fn((handler) => {
        // Store the handler for testing
        global.testStoredErrorHandler = handler;
        return { command: vi.fn().mockReturnThis() };
      }),
      command: vi.fn().mockReturnThis(),
      start: vi.fn().mockResolvedValue({}),
      stop: vi.fn().mockResolvedValue({}),
    })),
    GrammyError: class GrammyError extends Error {
      constructor(message: string) {
        super(message);
        this.name = "GrammyError";
      }
    },
    HttpError: class HttpError extends Error {
      constructor(message: string) {
        super(message);
        this.name = "HttpError";
      }
    },
  };
});

describe("TelegramService", () => {
  let telegramService: TelegramService;
  let mockLogger: LoggerInterface;
  const BOT_TOKEN = "test-token";
  const CHAT_ID = "12345";

  beforeEach(() => {
    // Create a mock logger
    mockLogger = {
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
      debug: vi.fn(),
      http: vi.fn(),
      verbose: vi.fn(),
      silly: vi.fn(),
      log: vi.fn(),
      child: vi.fn().mockReturnThis(),
    };

    // Create telegram service instance with a config object
    telegramService = new TelegramService(
      {
        botToken: BOT_TOKEN,
        chatId: CHAT_ID,
        logger: mockLogger,
      },
      { env: "test" } // Explicitly pass env for testing
    );
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("should be instantiated correctly", () => {
    expect(telegramService).toBeInstanceOf(TelegramService);
    expect(Bot).toHaveBeenCalledWith(BOT_TOKEN);
  });

  it("should send a message successfully", async () => {
    const message = "Test message";
    await telegramService.sendMessage(message);

    expect((telegramService as any).bot.api.sendMessage).toHaveBeenCalledWith(
      CHAT_ID,
      message,
      expect.any(Object)
    );
    expect(mockLogger.info).toHaveBeenCalled();
  });

  it("should send multiple messages successfully", async () => {
    const messages = ["Message 1", "Message 2"];
    for (const message of messages) {
      await telegramService.sendMessage(message);
    }

    expect((telegramService as any).bot.api.sendMessage).toHaveBeenCalledTimes(
      2
    );
    expect(mockLogger.info).toHaveBeenCalledTimes(2);
  });

  it("should register commands on instantiation", () => {
    // Bot instantiation happens in beforeEach
    const botInstance = (telegramService as any).bot;
    expect(botInstance.command).toHaveBeenCalledWith(
      "start",
      expect.any(Function)
    );
    expect(botInstance.command).toHaveBeenCalledWith(
      "help",
      expect.any(Function)
    );
    expect(botInstance.command).toHaveBeenCalledWith(
      "status",
      expect.any(Function)
    );
    expect(botInstance.command).toHaveBeenCalledWith(
      "opportunities",
      expect.any(Function)
    );
    expect(botInstance.command).toHaveBeenCalledWith(
      "settings",
      expect.any(Function)
    );
  });

  it("should send opportunity notification successfully", async () => {
    const opportunity: ArbitrageOpportunity = {
      id: "test-id-success",
      type: "fundingRate",
      pairSymbol: "BTC/USDT",
      timestamp: Date.now(),
      longExchange: "binance",
      shortExchange: "bybit",
      longRate: 0.01,
      shortRate: -0.02,
      grossProfitMetric: 0.03,
    };

    // Spy on formatOpportunityMessage
    const formatSpy = vi
      .spyOn(formatterUtils, "formatOpportunityMessage")
      .mockImplementation(() => "Formatted message");

    await telegramService.sendOpportunityNotification(opportunity);

    expect(formatSpy).toHaveBeenCalledWith(opportunity);
    expect((telegramService as any).bot.api.sendMessage).toHaveBeenCalledWith(
      CHAT_ID,
      "Formatted message",
      expect.objectContaining({ parse_mode: "MarkdownV2" })
    );
  });

  it("should handle error in sendOpportunityNotification and retry", async () => {
    const opportunity: ArbitrageOpportunity = {
      id: "test-id-retry",
      type: "fundingRate",
      pairSymbol: "BTC/USDT",
      timestamp: Date.now(),
      longExchange: "binance",
      shortExchange: "bybit",
      longRate: 0.01,
      shortRate: -0.02,
      grossProfitMetric: 0.03,
    };

    // Make the first attempt fail, then succeed
    vi.spyOn(formatterUtils, "formatOpportunityMessage").mockImplementation(
      () => "Formatted message"
    );

    const sendMessageMock = (telegramService as any).bot.api
      .sendMessage as MockedFunction<Bot<Context>["api"]["sendMessage"]>;
    sendMessageMock
      .mockRejectedValueOnce(new Error("Network error"))
      .mockResolvedValueOnce({
        message_id: 1,
        date: Math.floor(Date.now() / 1000),
        chat: {
          id: Number.parseInt(CHAT_ID, 10),
          type: "private",
          first_name: "Test Bot User",
        },
        text: "Mocked message text",
      } as Message.TextMessage);

    await telegramService.sendOpportunityNotification(opportunity);

    // Should have attempted twice (initial + 1 retry)
    expect(sendMessageMock).toHaveBeenCalledTimes(2);
    expect(mockLogger.error).toHaveBeenCalled();
    expect(mockLogger.info).toHaveBeenCalled();
  });

  describe("Error Handling via bot.catch", () => {
    it("should log GrammyError and attempt to reply", () => {
      if (global.testStoredErrorHandler) {
        const mockApiError: ApiError = {
          ok: false as const,
          error_code: 400,
          description: "Bad Request",
        };
        const grammyError = new GrammyError(
          "Test Grammy Error",
          mockApiError,
          "testMethod",
          {}
        );
        const mockCtx = {
          update: { update_id: 123 },
          from: { id: 1, is_bot: false, first_name: "Test" },
          chat: {
            id: 12345,
            type: "private" as const,
            first_name: "Test User",
          },
          reply: vi.fn().mockResolvedValue({
            message_id: 2,
            date: Math.floor(Date.now() / 1000),
            chat: {
              id: 12345,
              type: "private",
              first_name: "Test User From Reply",
            }, // Differentiate if needed
            text: "Mocked reply for GrammyError",
          } as Message.TextMessage), // More specific mock
        } as Partial<
          Context & {
            update: Update;
            reply: MockInstance<(text: string) => Promise<Message.TextMessage>>;
          }
        >;
        const handler = global.testStoredErrorHandler;
        if (handler) {
          handler({ error: grammyError, ctx: mockCtx });
        }

        expect(mockLogger.error).toHaveBeenCalledWith(
          expect.stringContaining("Error while handling update 123"),
          expect.objectContaining({ error: expect.any(String) })
        );
        expect(mockCtx.reply).toHaveBeenCalledWith(
          "An error occurred while processing your request. The team has been notified."
        );
      } else {
        throw new Error("Global error handler not set by mock Bot");
      }
    });

    it("should log HttpError and attempt to reply", () => {
      if (global.testStoredErrorHandler) {
        const httpError = new HttpError(
          "Test HTTP Error",
          new Error("Underlying network issue")
        );
        const mockCtx = {
          update: { update_id: 456 },
          from: { id: 1, is_bot: false, first_name: "Test" },
          chat: {
            id: 12345,
            type: "private" as const,
            first_name: "Test User",
          },
          reply: vi.fn().mockResolvedValue({
            message_id: 3,
            date: Math.floor(Date.now() / 1000),
            chat: {
              id: 12345,
              type: "private",
              first_name: "Test User From Reply 2",
            }, // Differentiate if needed
            text: "Mocked reply for HttpError",
          } as Message.TextMessage), // More specific mock
        } as Partial<
          Context & {
            update: Update;
            reply: MockInstance<(text: string) => Promise<Message.TextMessage>>;
          }
        >;
        const handler = global.testStoredErrorHandler;
        if (handler) {
          handler({ error: httpError, ctx: mockCtx });
        }

        expect(mockLogger.error).toHaveBeenCalledWith(
          expect.stringContaining("Error while handling update 456"),
          expect.objectContaining({ error: expect.any(String) })
        );
        expect(mockCtx.reply).toHaveBeenCalledWith(
          "An error occurred while processing your request. The team has been notified."
        );
      } else {
        throw new Error("Global error handler not set by mock Bot");
      }
    });
  });

  it("should stop the bot correctly", async () => {
    await telegramService.stop();
    expect((telegramService as any).bot.stop).toHaveBeenCalled();
  });
});
