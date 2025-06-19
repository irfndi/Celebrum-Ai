import type { 
  TelegramUpdate, 
  TelegramBotResponse, 
  TelegramHandler,
  TelegramWebhookContext 
} from '../types/index';

// Command handlers registry
export const TELEGRAM_HANDLERS = new Map<string, TelegramHandler>();

// Register a command handler
export function registerHandler(handler: TelegramHandler): void {
  TELEGRAM_HANDLERS.set(handler.command, handler);
}

// Get handler for command
export function getHandler(command: string): TelegramHandler | undefined {
  return TELEGRAM_HANDLERS.get(command);
}

// Process telegram update and route to appropriate handler
export async function processTelegramUpdate(
  update: TelegramUpdate,
  context: TelegramWebhookContext
): Promise<TelegramBotResponse | null> {
  try {
    // Extract command from message
    const message = update.message;
    if (!message?.text) {
      return null;
    }

    // Parse command (e.g., "/start", "/help")
    const commandMatch = message.text.match(/^\/(\w+)/);
    if (!commandMatch || !commandMatch[1]) {
      return null;
    }

    const command = commandMatch[1];
    const handler = getHandler(command);
    
    if (!handler) {
      return {
        method: 'sendMessage',
        chat_id: message.chat.id,
        text: `Unknown command: /${command}. Type /help for available commands.`,
        parse_mode: 'HTML'
      };
    }

    // Execute handler
    return await handler.handler(update, context.env);
  } catch (error) {
    console.error('Error processing telegram update:', error);
    
    // Return error response if we have a chat ID
    const chatId = update.message?.chat.id;
    if (chatId) {
      return {
        method: 'sendMessage',
        chat_id: chatId,
        text: '❌ An error occurred while processing your request. Please try again later.',
        parse_mode: 'HTML'
      };
    }
    
    return null;
  }
}

// Default handlers
export const defaultHandlers: TelegramHandler[] = [
  {
    command: 'start',
    description: 'Start the bot and show welcome message',
    handler: async (update: TelegramUpdate) => {
      const chatId = update.message?.chat.id;
      if (!chatId) return null;

      return {
        method: 'sendMessage',
        chat_id: chatId,
        text: `🚀 <b>Welcome to Celebrum Trading Platform!</b>

I'm your AI-powered trading assistant. Here's what I can help you with:

📊 <b>Market Analysis</b>
• Real-time arbitrage opportunities
• Price tracking across exchanges
• Market insights and trends

🛠️ <b>Trading Tools</b>
• Portfolio management
• Risk assessment
• Trade execution assistance

Type /help to see all available commands or /opportunities to get started!

<i>Ready to maximize your trading potential? Let's go! 🎯</i>`,
        parse_mode: 'HTML'
      };
    }
  },
  {
    command: 'help',
    description: 'Show available commands',
    handler: async (update: TelegramUpdate) => {
      const chatId = update.message?.chat.id;
      if (!chatId) return null;

      const commandList = Array.from(TELEGRAM_HANDLERS.values())
        .map(handler => `/${handler.command} - ${handler.description}`)
        .join('\n');

      return {
        method: 'sendMessage',
        chat_id: chatId,
        text: `📋 <b>Available Commands:</b>\n\n${commandList}`,
        parse_mode: 'HTML'
      };
    }
  }
];

// Initialize default handlers
export function initializeDefaultHandlers(): void {
  defaultHandlers.forEach(handler => {
    registerHandler(handler);
  });
} 