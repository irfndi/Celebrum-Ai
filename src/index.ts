// src/index.ts
import { Router } from "itty-router";
import { webhookCallback } from "grammy"; // Removed Bot import as it's not directly used here
import type { Env, LoggerInterface, TelegramConfig, StructuredTradingPair, ExchangeId, TradingPairSymbol } from "./types"; // Added necessary types
import type { ExecutionContext, ScheduledController } from "@cloudflare/workers-types"; // Added ScheduledController
import { TelegramService } from "./services/telegramService";
import { ExchangeService } from "./services/exchangeService";
import { OpportunityService } from "./services/opportunityService";
import type { OpportunityServiceConfig } from "./services/opportunityService"; // Added import
import { createLogger } from "./utils/logger";

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
    const logger = env.LOGGER || createLogger(env.LOG_LEVEL || 'info');
    try {
      let telegramService: TelegramService;

      if (env.telegramServiceInstance) {
        logger.info("Webhook: Using existing TelegramService instance from env.");
        telegramService = env.telegramServiceInstance;
      } else {
        logger.info("Webhook: Creating new TelegramService instance.");
        if (!env.TELEGRAM_BOT_TOKEN || !env.TELEGRAM_CHAT_ID) {
          logger.error("Telegram secrets not configured in environment.");
          return new Response("Telegram secrets not configured", { status: 500 });
        }
        const tgConfig: TelegramConfig = {
          botToken: env.TELEGRAM_BOT_TOKEN,
          chatId: env.TELEGRAM_CHAT_ID,
          logger: logger as LoggerInterface,
        };
        telegramService = new TelegramService(tgConfig, {
          startPolling: false,
          env: "production_webhook", // Consistent with original
        });
      }

      const botInstance = telegramService.getBotInstance();
      const handleUpdate = webhookCallback(botInstance, "cloudflare-mod");
      return await handleUpdate(request);
    } catch (e: unknown) {
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
      logger.error("Webhook processing error (unknown type):", e);
      return new Response(
        "Internal Server Error: An unexpected error occurred",
        { status: 500 }
      );
    }
  }
);

// --- BEGIN NEW /find-opportunities ROUTE ---
router.post(
  "/find-opportunities",
  async (request: Request, env: Env, ctx: ExecutionContext) => {
    console.log("@@@ POST /find-opportunities ROUTE HIT @@@");
    const logger = env.LOGGER || createLogger(env.LOG_LEVEL || 'info');
    logger.info("POST /find-opportunities request received");

    try {
      const exchangeIdsString = env.EXCHANGES;
      if (!exchangeIdsString || exchangeIdsString.split(",").length < 2) {
        logger.error(
          "At least two exchanges must be configured in EXCHANGES env var for /find-opportunities."
        );
        return new Response(
          JSON.stringify({
            status: "error",
            message: "Exchange configuration error: At least two exchanges must be configured.",
          }),
          { status: 400, headers: { "Content-Type": "application/json" } }
        );
      }
      const exchangeIds = exchangeIdsString.split(",") as ExchangeId[];

      if (!env.MONITORED_PAIRS_CONFIG) {
        logger.error("MONITORED_PAIRS_CONFIG is not set in env for /find-opportunities.");
        return new Response(
          JSON.stringify({
            status: "error",
            message: "Configuration error: MONITORED_PAIRS_CONFIG is not set.",
          }),
          { status: 400, headers: { "Content-Type": "application/json" } }
        );
      }

      let monitoredPairsStruct: StructuredTradingPair[];
      try {
        monitoredPairsStruct = JSON.parse(env.MONITORED_PAIRS_CONFIG);
        if (!Array.isArray(monitoredPairsStruct) || monitoredPairsStruct.length === 0) {
          logger.error("MONITORED_PAIRS_CONFIG is not a valid non-empty array for /find-opportunities.");
          return new Response(
            JSON.stringify({
              status: "error",
              message: "Configuration error: MONITORED_PAIRS_CONFIG is invalid.",
            }),
            { status: 400, headers: { "Content-Type": "application/json" } }
          );
        }
      } catch (e: unknown) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        logger.error(
          "Failed to parse MONITORED_PAIRS_CONFIG from env for /find-opportunities:",
          errorMsg,
          e
        );
        return new Response(
          JSON.stringify({
            status: "error",
            message: `Configuration error: Failed to parse MONITORED_PAIRS_CONFIG: ${errorMsg}`,
          }),
          { status: 400, headers: { "Content-Type": "application/json" } }
        );
      }
      const monitoredPairSymbols: TradingPairSymbol[] = monitoredPairsStruct.map(p => p.symbol);

      const exchangeService = new ExchangeService({ env, logger: logger as LoggerInterface });

      let telegramServiceInstance: TelegramService | null = null;
      if (env.TELEGRAM_BOT_TOKEN && env.TELEGRAM_CHAT_ID) {
        telegramServiceInstance = new TelegramService(
          {
            botToken: env.TELEGRAM_BOT_TOKEN,
            chatId: env.TELEGRAM_CHAT_ID,
            logger: logger as LoggerInterface,
          },
          { startPolling: false, env: "production_http" } // Indicate HTTP context
        );
      }

      const opportunityServiceConfig = {
        exchangeService,
        telegramService: telegramServiceInstance,
        logger: logger as LoggerInterface,
        monitoredPairs: monitoredPairsStruct,
        exchanges: exchangeIds,
        threshold: Number.parseFloat(env.ARBITRAGE_THRESHOLD || "0.001"),
      };
      const opportunityService = new OpportunityService(opportunityServiceConfig);

      logger.info("/find-opportunities: Finding opportunities...");
      const opportunities = await opportunityService.findOpportunities(
        exchangeIds,
        monitoredPairSymbols,
        opportunityServiceConfig.threshold
      );

      if (opportunities.length > 0) {
        logger.info(`/find-opportunities: Found ${opportunities.length} opportunities`);
        if (telegramServiceInstance) {
          // Send notifications for each opportunity without awaiting them individually in the main flow
          // Use waitUntil to allow these to complete after the response is sent
          for (const opportunity of opportunities) {
            try {
              // Important: Do not await here directly if you want to send response fast.
              // The promise from sendOpportunityNotification will be handled by waitUntil.
              const notificationPromise = telegramServiceInstance.sendOpportunityNotification(opportunity)
                .then(() => {
                  logger.info("Successfully sent Telegram notification for an opportunity.", opportunity);
                })
                .catch(notificationError => {
                  logger.error("Failed to send Telegram notification for an opportunity", notificationError, opportunity);
                });
              ctx.waitUntil(notificationPromise);
            } catch (syncError) {
              // Catch synchronous errors from calling sendOpportunityNotification or ctx.waitUntil itself
              logger.error("Synchronous error during telegramService.sendOpportunityNotification or ctx.waitUntil call", syncError, opportunity);
            }
          }
        } else {
          logger.warn("Opportunities found, but no TelegramService available to send notifications.");
        }
      } else {
        logger.info("/find-opportunities: No opportunities found.");
      }

      return new Response(
        JSON.stringify({
          status: "success",
          opportunitiesFound: opportunities.length,
          opportunities,
        }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      );
    } catch (error: unknown) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      logger.error(
        `Error in /find-opportunities route: ${errorMsg}`,
        error
      );
      return new Response(
        JSON.stringify({
          status: "error",
          message: `Error finding opportunities: ${errorMsg}`,
        }),
        { status: 500, headers: { "Content-Type": "application/json" } }
      );
    }
  }
);
// --- END NEW /find-opportunities ROUTE ---

// Catch-all for 404s
// Modified to return JSON for consistency in error handling, as tests expect JSON parsing.
router.all("*", () => {
  console.log("@@@ CATCH-ALL ROUTE * HIT @@@");
  return new Response(JSON.stringify({ status: "error", message: "Route not found" }), { 
    status: 404, 
    headers: { "Content-Type": "application/json" }
  });
});

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext
  ): Promise<Response> {
    if (!env.LOGGER) {
      // Ensure logger is available in env for router handlers if they rely on it.
      // The createLogger call here might be too late if router handler accesses env.LOGGER before this fetch function sets it.
      // Consider initializing logger earlier or ensuring all handlers can create one if not present.
      env.LOGGER = createLogger(env.LOG_LEVEL || 'info');
    }
    return router.fetch(request, env, ctx);
  },
  // We'll add the scheduled handler later if needed by TelegramService or other services
  async scheduled(
    controller: ScheduledController,
    env: Env,
    ctx: ExecutionContext
  ): Promise<void> {
    const logger = env.LOGGER || createLogger(env.LOG_LEVEL || 'info');
    logger.info(`Scheduled handler triggered by cron: ${controller.cron} at ${new Date(controller.scheduledTime).toISOString()}`);

    try {
      let opportunityServiceToUse: OpportunityService;
      let exchangeIdsToUse: ExchangeId[];
      let monitoredPairSymbolsToUse: TradingPairSymbol[];
      let thresholdToUse: number;
      let telegramServiceForNotification: TelegramService | null = null;
      let monitoredPairsStructForNewInstance: StructuredTradingPair[] = []; // Used only if creating new OppService

      if (env.opportunityServiceInstance) {
        logger.info("Scheduled: Using provided OpportunityService instance.");
        opportunityServiceToUse = env.opportunityServiceInstance;
        
        const config = opportunityServiceToUse.getConfig(); // Assume this returns OpportunityServiceConfig
        exchangeIdsToUse = config.exchanges;
        monitoredPairSymbolsToUse = config.monitoredPairs.map((p: StructuredTradingPair) => p.symbol);
        thresholdToUse = config.threshold;
        telegramServiceForNotification = config.telegramService; // Can be null

        if (!exchangeIdsToUse || exchangeIdsToUse.length < 2) {
           logger.error("Scheduled: Insufficient exchanges from provided OpportunityService config.");
           return;
        }
        if (!monitoredPairSymbolsToUse || monitoredPairSymbolsToUse.length === 0) {
            logger.error("Scheduled: No monitored pairs from provided OpportunityService config.");
            return;
        }
      } else {
        logger.info("Scheduled: Creating new OpportunityService instance.");
        const exchangeIdsString = env.EXCHANGES;
        if (!exchangeIdsString || exchangeIdsString.split(",").length < 2) {
          logger.error(
            "At least two exchanges must be configured in EXCHANGES env var for the scheduled task."
          );
          return;
        }
        exchangeIdsToUse = exchangeIdsString.split(",") as ExchangeId[];

        if (!env.MONITORED_PAIRS_CONFIG) {
          logger.error("MONITORED_PAIRS_CONFIG is not set in env for the scheduled task.");
          return;
        }
        try {
          monitoredPairsStructForNewInstance = JSON.parse(env.MONITORED_PAIRS_CONFIG);
          if (!Array.isArray(monitoredPairsStructForNewInstance) || monitoredPairsStructForNewInstance.length === 0) {
            logger.error("MONITORED_PAIRS_CONFIG is not a valid non-empty array.");
            return;
          }
        } catch (e: unknown) {
          const errorMsg = e instanceof Error ? e.message : String(e);
          logger.error("Failed to parse MONITORED_PAIRS_CONFIG from env for scheduled task:", errorMsg, e);
          return;
        }
        monitoredPairSymbolsToUse = monitoredPairsStructForNewInstance.map(p => p.symbol);
        thresholdToUse = Number.parseFloat(env.ARBITRAGE_THRESHOLD || "0.001");

        const exchangeService = new ExchangeService({ env, logger: logger as LoggerInterface });
        
        if (env.TELEGRAM_BOT_TOKEN && env.TELEGRAM_CHAT_ID) {
          telegramServiceForNotification = new TelegramService({
            botToken: env.TELEGRAM_BOT_TOKEN,
            chatId: env.TELEGRAM_CHAT_ID,
            logger: logger as LoggerInterface,
          }, { startPolling: false, env: "production_scheduled" });
        }

        const opportunityServiceConfig: OpportunityServiceConfig = {
          exchangeService,
          telegramService: telegramServiceForNotification,
          logger: logger as LoggerInterface,
          monitoredPairs: monitoredPairsStructForNewInstance, 
          exchanges: exchangeIdsToUse,
          threshold: thresholdToUse,
        };
        opportunityServiceToUse = new OpportunityService(opportunityServiceConfig);
      }

      logger.info("Scheduled: Finding opportunities...");
      const opportunities = await opportunityServiceToUse.findOpportunities(
        exchangeIdsToUse,
        monitoredPairSymbolsToUse,
        thresholdToUse
      );
      logger.info(`Scheduled: Found ${opportunities.length} opportunities.`);

      if (opportunities.length > 0 && telegramServiceForNotification) {
        for (const opp of opportunities) {
          ctx.waitUntil(telegramServiceForNotification.sendOpportunityNotification(opp)
            .catch(err => logger.error("Scheduled: Failed to send Telegram notification:", err, opp))
          );
        }
      } else if (opportunities.length > 0) {
        logger.warn("Scheduled: Opportunities found, but Telegram service not configured/available to send notifications.");
      }
      
      // logger.info("UNIQUE_DEBUG_LOG_POINT_BEFORE_FINAL_LOGS_IN_SCHEDULED"); // <-- REMOVED THIS
      // Added to satisfy test expectation from tests/index.worker.scheduled.test.ts
      logger.info("Cron job: findOpportunities completed successfully."); 
      logger.info("Scheduled task processed successfully.");


    } catch (error: unknown) {
      if (error instanceof Error) {
        logger.error("Error in scheduled handler execution:", error.message, error);
      } else {
        logger.error("Error in scheduled handler execution (unknown error type):", error);
      }
    }
  }
};
