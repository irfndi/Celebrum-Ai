import { Bot, GrammyError, HttpError } from 'grammy';
import type { ArbitrageOpportunity } from '../types'; // Use type import if only used as type
import { formatOpportunityMessage } from '../utils/formatter';

// Interface for configuration (can be expanded)
interface TelegramConfig {
  botToken: string;
  chatId: string; // Target chat ID to send notifications
}

export class TelegramService {
  private bot: Bot;
  private config: TelegramConfig;

  constructor(config: TelegramConfig) {
    if (!config.botToken || !config.chatId) {
      throw new Error('Telegram Bot Token and Chat ID must be provided.');
    }
    this.config = config;
    this.bot = new Bot(this.config.botToken);

    // TODO: Add command handlers (start, help, execute_trade, etc.) later

    // Basic error handling
    this.bot.catch((err) => {
      const ctx = err.ctx;
      console.error(`Error while handling update ${ctx.update.update_id}:`);
      const e = err.error;
      // TODO: Add more robust error logging/handling
      console.error('Error details:', e);
    });

    // Basic command (example)
    this.bot.command('start', (ctx) => ctx.reply('Arbitrage Bot Started! Use /help for commands.'));

    // Start the bot (in polling mode for now, consider webhooks for CF Workers)
    // Only start polling if not in a test environment or specific config
    if (process.env.NODE_ENV !== 'test') {
      this.bot.start();
      console.log('Telegram Bot started in polling mode...');
    }
  }

  /**
   * Sends a notification message about a detected arbitrage opportunity.
   * @param opportunity The detected arbitrage opportunity.
   */
  async sendOpportunityNotification(opportunity: ArbitrageOpportunity): Promise<void> {
    try {
      const message = formatOpportunityMessage(opportunity);
      await this.bot.api.sendMessage(this.config.chatId, message, {
        parse_mode: 'MarkdownV2', // Use Markdown for formatting
      });
      console.log(`Sent notification for ${opportunity.pair} to chat ${this.config.chatId}`);
    } catch (error) {
      console.error(`Failed to send Telegram notification for ${opportunity.pair}:`, error);
      // TODO: Implement retry logic or better error reporting
    }
  }

  // --- Other methods for handling commands, etc. will go here ---

  /**
   * Gracefully stops the bot.
   */
  async stop(): Promise<void> {
    if (process.env.NODE_ENV !== 'test') {
       await this.bot.stop();
       console.log('Telegram Bot stopped.');
    }
  }
}

// Example usage (consider moving to main application logic)
// Ensure environment variables are loaded (e.g., using dotenv)
// import dotenv from 'dotenv';
// dotenv.config();
//
// const telegramConfig: TelegramConfig = {
//   botToken: process.env.TELEGRAM_BOT_TOKEN || '',
//   chatId: process.env.TELEGRAM_CHAT_ID || '',
// };
//
// if (telegramConfig.botToken && telegramConfig.chatId) {
//   const telegramService = new TelegramService(telegramConfig);
//   // Example notification
//   // const fakeOpp: ArbitrageOpportunity = {
//   //   pair: 'BTC/USDT',
//   //   longExchange: 'bybit',
//   //   shortExchange: 'binance',
//   //   longRate: 0.0001,
//   //   shortRate: 0.0010,
//   //   rateDifference: 0.0009,
//   //   timestamp: Date.now(),
//   // };
//   // telegramService.sendOpportunityNotification(fakeOpp);
// }
