// src/index.ts
import { Router } from 'itty-router';

// --- TODO: Import your services and DO classes ---
// Example:
// import { handleOpportunityCheck } from './services/opportunityService';
// import { handleTelegramCommand } from './services/telegramService';
import { PositionsManager } from './services/positionsManager'; // Import the DO class

/**
 * Defines the environment bindings expected by the Worker.
 * Add secrets, KV namespaces, Durable Objects, etc. here.
 */
export interface Env {
	// KV
	ArbEdgeKV: KVNamespace;

	// Durable Objects
	POSITIONS: DurableObjectNamespace;

	// Secrets (Add all secrets expected by your functions)
	BYBIT_API_KEY: string;
	BYBIT_API_SECRET: string;
	BINANCE_API_KEY: string;
	BINANCE_API_SECRET: string;
	TELEGRAM_BOT_TOKEN: string;
	TELEGRAM_CHAT_ID: string; // If needed as a secret

	// Variables (from wrangler.toml)
	USDT_AMOUNT: string;
	BYBIT_LEVERAGE: string;
	BINANCE_LEVERAGE: string;
	ENVIRONMENT: string;
}

const router = Router();

// --- TODO: Define your actual routes --- 
// Example route
router.get('/', () => new Response('ArbEdge Worker is running!\n\nRoutes:\n - / [GET] : This message\n - Add other routes here...'));

// Example: Route for checking opportunities (modify as needed)
// router.post('/check-opportunities', handleOpportunityCheck);

// Example: Route for Telegram webhook (modify as needed)
// router.post('/telegram/webhook/:token', async (request, env: Env) => {
//   if (request.params.token !== env.TELEGRAM_BOT_TOKEN) {
//     return new Response('Invalid token', { status: 401 });
//   }
//   // return handleTelegramCommand(request, env);
//   return new Response('Telegram handler not implemented yet');
// });

// Catch-all for undefined routes
router.all('*', () => new Response('Not Found.', { status: 404 }));

export default {
	/**
	 * Handles incoming HTTP requests.
	 */
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		console.log(`[${env.ENVIRONMENT}] Request: ${request.method} ${request.url}`);
		// Pass env and ctx to router handlers if they need access
		return router.handle(request, env, ctx);
	},

	/**
	 * Handles scheduled events based on cron triggers in wrangler.toml.
	 */
	async scheduled(controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
	  console.log(`[${env.ENVIRONMENT}] Cron Trigger: ${controller.cron} at ${new Date(controller.scheduledTime)}`);
	  try {
	    // Example: Trigger opportunity check
	    // import { OpportunityService } from './services/opportunityService'; // Make sure to import if used
	    // const opportunityService = new OpportunityService(env);
	    // await opportunityService.findAndExecuteArbitrage();
	    console.log('Scheduled task executed (placeholder)');
	  } catch (error) {
	    console.error('Error during scheduled task:', error);
	  }
	  // Ensure the task runs to completion, even if the fetch handler returns early.
	  // ctx.waitUntil(Promise.resolve()); // waitUntil typically takes a promise to wait for, not just Promise.resolve()
	},
};

// --- Export the Durable Object class so Wrangler can bind it ---
export { PositionsManager };
