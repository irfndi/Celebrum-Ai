// src/index.ts
import { Router } from "itty-router";
import { webhookCallback, Bot } from "grammy"; // Import Bot for type, webhookCallback
import type { Env, LoggerInterface, TelegramConfig } from "./types";
import type { ExecutionContext } from "@cloudflare/workers-types";
import { TelegramService } from "./services/telegramService";
// import { createLogger } from "./utils/logger"; // Assuming logger is on env

const router = Router();

// --- Telegram Bot Setup ---
// We need to initialize the TelegramService (and the bot within it) once
// so it can be used by the webhook route and potentially other parts of the application.
// However, env is only available inside the fetch handler.
// This means we might need to create the bot instance or the service
// per-request or use a global/cached instance if env variables are constant
// and the logger can be created/obtained.

// For now, let's create it inside the route that needs it, or explore
// how to initialize it once at the top level of the module if env allows.

// If TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID are available globally (e.g. from process.env for non-worker envs)
// or can be set up during worker initialization, we could do this:
/*
let telegramServiceInstance: TelegramService | null = null;

function getTelegramService(env: Env): TelegramService {
  if (!telegramServiceInstance) {
    const logger = env.LOGGER || createLogger({ level: env.LOG_LEVEL || 'info' }); // Get or create logger
    const tgConfig: TelegramConfig = {
      botToken: env.TELEGRAM_BOT_TOKEN,
      chatId: env.TELEGRAM_CHAT_ID,
      logger: logger as LoggerInterface,
    };
    telegramServiceInstance = new TelegramService(tgConfig, { startPolling: false }); // Ensure polling is off for webhook
  }
  return telegramServiceInstance;
}
*/
// This caching strategy has issues in CF workers if env is request-specific and not truly global.
// For CF Workers, env is passed to fetch, so service should be instantiated there or use a DO.

// --- Routes ---

router.get(
  "/ping",
  () => new Response("pong", { headers: { "Content-Type": "text/plain" } })
);

router.get(
  "/ping-direct",
  () =>
    new Response("pong direct", { headers: { "Content-Type": "text/plain" } })
);

router.post(
  "/webhook",
  async (request: Request, env: Env, ctx: ExecutionContext) => {
    const logger = env.LOGGER || console; // Define logger early for use in catch
    try {
      if (!env.TELEGRAM_BOT_TOKEN || !env.TELEGRAM_CHAT_ID) {
        logger.error("Telegram secrets not configured in environment.");
        return new Response("Telegram secrets not configured", { status: 500 });
      }

      const tgConfig: TelegramConfig = {
        botToken: env.TELEGRAM_BOT_TOKEN,
        chatId: env.TELEGRAM_CHAT_ID,
        logger: logger as LoggerInterface,
      };

      const telegramService = new TelegramService(tgConfig, {
        startPolling: false,
        env: "production_webhook",
      });
      const botInstance = telegramService.getBotInstance();
      const handleUpdate = webhookCallback(botInstance, "cloudflare-mod");
      return handleUpdate(request);
    } catch (e: unknown) {
      // Simplified catch block. If grammy throws, it will likely be an Error.
      // Log the error and return a generic 500.
      // This means bad client requests (malformed JSON, wrong content type)
      // will also result in a 500 if grammy throws a generic error for them.
      logger.error("Webhook caught error object:", e);
      if (e instanceof Error) {
        logger.error(
          "Webhook processing error (Error instance):",
          e.message,
          e.stack
        );
        return new Response(`Internal Server Error: ${e.message}`, {
          status: 500,
        });
      }
      // Fallback for non-Error objects thrown
      logger.error("Webhook processing error (unknown type):", e);
      return new Response(
        "Internal Server Error: An unexpected error occurred",
        { status: 500 }
      );
    }
  }
);

// Catch-all for 404s
router.all("*", () => new Response("ONLY ALL ROUTE ACTIVE", { status: 404 }));

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext
  ): Promise<Response> {
    // Optional: Could instantiate services that need env here and pass them down
    // if they are used by multiple routes.
    // For example:
    // if (!env.LOGGER && createLogger) { // Assuming createLogger is available
    //   env.LOGGER = createLogger({ level: env.LOG_LEVEL || 'info', service: 'worker-fetch' });
    // }
    return router.fetch(request, env, ctx);
  },
  // We'll add the scheduled handler later if needed by TelegramService or other services
};
