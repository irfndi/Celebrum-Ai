import type { 
  TelegramBotResponse, 
  TelegramIntegrationConfig,
  TelegramUpdate 
} from '../types/index';

// Telegram API utilities
export class TelegramAPI {
  private baseUrl: string;

  constructor(botToken: string) {
    this.baseUrl = `https://api.telegram.org/bot${botToken}`;
  }

  // Send message to Telegram
  async sendMessage(
    chatId: number, 
    text: string, 
    options: Partial<TelegramBotResponse> = {}
  ): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/sendMessage`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chat_id: chatId,
          text,
          parse_mode: 'HTML',
          ...options
        })
      });

      return response.ok;
    } catch (error) {
      console.error('Failed to send telegram message:', error);
      return false;
    }
  }

  // Set webhook
  async setWebhook(url: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/setWebhook`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          url,
          allowed_updates: ['message', 'callback_query']
        })
      });

      return response.ok;
    } catch (error) {
      console.error('Failed to set telegram webhook:', error);
      return false;
    }
  }

  // Delete webhook
  async deleteWebhook(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/deleteWebhook`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        }
      });

      return response.ok;
    } catch (error) {
      console.error('Failed to delete telegram webhook:', error);
      return false;
    }
  }

  // Get webhook info
  async getWebhookInfo(): Promise<any> {
    try {
      const response = await fetch(`${this.baseUrl}/getWebhookInfo`);
      return await response.json();
    } catch (error) {
      console.error('Failed to get webhook info:', error);
      return null;
    }
  }
}

// Validation utilities
export function validateTelegramUpdate(data: any): data is TelegramUpdate {
  return (
    typeof data === 'object' &&
    data !== null &&
    typeof data.update_id === 'number'
  );
}

export function validateBotToken(token: string): boolean {
  // Telegram bot tokens are in format: <bot_id>:<auth_token>
  // Example: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz
  const tokenRegex = /^\d+:[A-Za-z0-9_-]{35}$/;
  return tokenRegex.test(token);
}

// Configuration utilities
export function createTelegramConfig(env: any): TelegramIntegrationConfig | null {
  const botToken = env.TELEGRAM_BOT_TOKEN;
  const webhookUrl = env.TELEGRAM_WEBHOOK_URL;

  if (!botToken || !validateBotToken(botToken)) {
    console.error('Invalid or missing TELEGRAM_BOT_TOKEN');
    return null;
  }

  if (!webhookUrl) {
    console.error('Missing TELEGRAM_WEBHOOK_URL');
    return null;
  }

  return {
    botToken,
    webhookUrl,
    allowedUsers: env.TELEGRAM_ALLOWED_USERS ? 
      env.TELEGRAM_ALLOWED_USERS.split(',').map(Number) : undefined,
    adminUsers: env.TELEGRAM_ADMIN_USERS ? 
      env.TELEGRAM_ADMIN_USERS.split(',').map(Number) : undefined
  };
}

// Response utilities
export function createSuccessResponse(message: string = 'OK'): Response {
  return new Response(JSON.stringify({ ok: true, message }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' }
  });
}

export function createErrorResponse(error: string, status: number = 400): Response {
  return new Response(JSON.stringify({ ok: false, error }), {
    status,
    headers: { 'Content-Type': 'application/json' }
  });
}

// Text formatting utilities
export function escapeHTML(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function formatPrice(price: number, decimals: number = 4): string {
  return price.toFixed(decimals);
}

export function formatPercentage(value: number, decimals: number = 2): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

// Logging utilities
export function logTelegramEvent(event: string, data: any): void {
  console.log(`[TELEGRAM] ${event}:`, JSON.stringify(data, null, 2));
}

export function logTelegramError(error: string, details?: any): void {
  console.error(`[TELEGRAM ERROR] ${error}`, details ? JSON.stringify(details, null, 2) : '');
} 