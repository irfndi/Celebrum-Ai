import { Hono } from 'hono';
import type { Env } from '@celebrum-ai/shared';
import { UserService, SessionService } from '@celebrum-ai/shared';
import type { User, NewUser } from '@celebrum-ai/db/schema/users';
import { sql } from 'drizzle-orm';

/**
 * ServiceRouter handles routing requests to appropriate services
 * based on URL patterns and service availability
 */
export class ServiceRouter {

  
  /**
   * Handle API requests
   * Routes to core API service
   */
  apiHandler() {
    const apiApp = new Hono<{ Bindings: Env }>();
    
    // All API routes
    apiApp.all('/*', async (c) => {
      const url = new URL(c.req.url);
      let path = url.pathname;
      
      // Normalize API paths
      if (path.startsWith('/api/')) {
        path = path.replace('/api', '');
      } else if (path.startsWith('/v1/')) {
        path = path.replace('/v1', '/api/v1');
      }
      
      // Route to API service
      if (c.env.API_SERVICE_URL) {
        const targetUrl = new URL(path + url.search, c.env.API_SERVICE_URL);
        const fetchOptions: RequestInit = {
          method: c.req.method,
          headers: c.req.header(),
        };
        
        if (c.req.method !== 'GET' && c.req.method !== 'HEAD') {
          fetchOptions.body = await c.req.arrayBuffer();
        }
        
        return fetch(targetUrl.toString(), fetchOptions);
      }
      
      // Fallback: basic API response
      return c.json({
        service: 'celebrum-ai-api',
        version: '1.0.0',
        path: path,
        method: c.req.method,
        timestamp: new Date().toISOString(),
        message: 'API service routing placeholder',
      });
    });
    
    return apiApp;
  }
  
  /**
   * Handle Telegram bot webhook requests
   */
  telegramBotHandler() {
    const telegramApp = new Hono<{ Bindings: Env }>();
    
    telegramApp.post('/webhook', async (c) => {
      try {
        const update = await c.req.json();
        
        // Handle different types of updates
        if (update.message) {
          return await this.handleMessage(c, update.message);
        }
        
        if (update.callback_query) {
          return await this.handleCallbackQuery(c, update.callback_query);
        }
        
        return c.json({ ok: true });
      } catch (error) {
        console.error('Error processing Telegram webhook:', error);
        return c.json({ ok: false, error: 'Internal server error' }, 500);
      }
    });
    
    return telegramApp;
  }
  
  /**
   * Handle incoming Telegram messages
   */
  private async handleMessage(c: any, message: any) {
    const userService = new UserService(c.env.DB);
    const sessionService = new SessionService(c.env.SESSIONS);
    
    const telegramUser = message.from;
    const text = message.text;
    
    // Handle /start command
    if (text === '/start') {
      return await this.handleStartCommand(c, telegramUser, userService, sessionService);
    }
    
    // For other commands, ensure user exists and has session
    const user = await userService.findUserByTelegramId(telegramUser.id.toString());
    if (!user) {
      return await this.sendMessage(c, telegramUser.id, 
        'Welcome! Please use /start to begin using ArbEdge.');
    }
    
    // Create or refresh session
    const session = await sessionService.createSession(user);
    
    // Handle other commands here
    return c.json({ ok: true });
  }
  
  /**
   * Handle /start command - create or update user profile and session
   */
  private async handleStartCommand(c: any, telegramUser: any, userService: UserService, sessionService: SessionService) {
    try {
      // Check if user already exists
      let user = await userService.findUserByTelegramId(telegramUser.id.toString());
      
      if (user) {
        // Update existing user with latest Telegram info
        const updates: Partial<NewUser> = {
          firstName: telegramUser.first_name,
          lastName: telegramUser.last_name,
          username: telegramUser.username,
          languageCode: telegramUser.language_code,
          lastActiveAt: new Date()
        };
        
        user = await userService.updateUser(user.id, updates);
        
        if (!user) {
          throw new Error('Failed to update user');
        }
        
        // Create new session
        const session = await sessionService.createSession(user);
        
        // Send welcome back message
        await this.sendMessage(c, telegramUser.id, 
          `Welcome back, ${user.firstName || user.username || 'User'}! 🎯\n\n` +
          `Your ArbEdge account is active.\n` +
          `Role: ${user.role.toUpperCase()}\n` +
          `Status: ${user.status}\n\n` +
          `Ready to find arbitrage opportunities? Use /help to see available commands.`);
        
      } else {
        // Create new user
        const newUserData: Partial<NewUser> = {
          telegramId: telegramUser.id.toString(),
          firstName: telegramUser.first_name,
          lastName: telegramUser.last_name,
          username: telegramUser.username,
          languageCode: telegramUser.language_code,
          role: 'free',
          status: 'active',
          lastActiveAt: new Date(),
          settings: {
            notifications: true,
            theme: 'light',
            language: telegramUser.language_code || 'en'
          },
          apiLimits: {
            exchangeApis: 2,
            aiApis: 10,
            maxDailyRequests: 100
          },
          tradingPreferences: {
            percentagePerTrade: 5,
            maxConcurrentTrades: 3,
            riskTolerance: 'medium',
            autoTrade: false
          }
        };
        
        user = await userService.createUser(newUserData);
        
        // Create initial session
        const session = await sessionService.createSession(user);
        
        // Send welcome message for new user
        await this.sendMessage(c, telegramUser.id, 
          `Welcome to ArbEdge! 🚀\n\n` +
          `Your account has been created successfully.\n` +
          `Role: FREE (upgrade available)\n` +
          `Daily API Requests: 100\n\n` +
          `ArbEdge helps you find and execute arbitrage opportunities across multiple exchanges.\n\n` +
          `Use /help to see available commands or /profile to view your settings.`);
      }
      
      return c.json({ ok: true });
      
    } catch (error) {
      console.error('Error in handleStartCommand:', error);
      await this.sendMessage(c, telegramUser.id, 
        'Sorry, there was an error setting up your account. Please try again later.');
      return c.json({ ok: false, error: 'Failed to process start command' });
    }
  }
  
  /**
   * Handle callback queries (inline keyboard responses)
   */
  private async handleCallbackQuery(c: any, callbackQuery: any) {
    // Handle inline keyboard callbacks here
    return c.json({ ok: true });
  }
  
  /**
   * Send message to Telegram user
   */
  private async sendMessage(c: any, chatId: number, text: string, replyMarkup?: any) {
    const botToken = c.env.TELEGRAM_BOT_TOKEN;
    if (!botToken) {
      console.error('TELEGRAM_BOT_TOKEN not configured');
      return;
    }
    
    const url = `https://api.telegram.org/bot${botToken}/sendMessage`;
    const payload: any = {
      chat_id: chatId,
      text: text,
      parse_mode: 'HTML'
    };
    
    if (replyMarkup) {
      payload.reply_markup = replyMarkup;
    }
    
    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      });
      
      if (!response.ok) {
        console.error('Failed to send Telegram message:', await response.text());
      }
    } catch (error) {
      console.error('Error sending Telegram message:', error);
    }
  }
  
  /**
   * Handle Discord bot webhook requests
   */
  discordBotHandler() {
    const discordApp = new Hono<{ Bindings: Env }>();
    
    discordApp.all('/*', async (c) => {
      const url = new URL(c.req.url);
      const path = url.pathname.replace('/webhook/discord', '');
      
      // Route to Discord bot service
      if (c.env.DISCORD_BOT_SERVICE_URL) {
        const targetUrl = new URL(path + url.search, c.env.DISCORD_BOT_SERVICE_URL);
        const fetchOptions: RequestInit = {
          method: c.req.method,
          headers: c.req.header(),
        };
        
        if (c.req.method !== 'GET' && c.req.method !== 'HEAD') {
          fetchOptions.body = await c.req.arrayBuffer();
        }
        
        return fetch(targetUrl.toString(), fetchOptions);
      }
      
      // Fallback: acknowledge webhook
      return c.json({
        ok: true,
        service: 'discord-bot',
        message: 'Webhook received',
        timestamp: new Date().toISOString(),
      });
    });
    
    return discordApp;
  }
  
  /**
   * Handle admin panel requests
   */
  adminHandler() {
    const adminApp = new Hono<{ Bindings: Env }>();
    
    adminApp.all('/*', async (c) => {
      const url = new URL(c.req.url);
      const path = url.pathname.replace('/admin', '');
      
      // Basic admin authentication check
      const authHeader = c.req.header('Authorization');
      if (!authHeader || !this.isValidAdminAuth(authHeader)) {
        return c.json({ error: 'Unauthorized' }, 401);
      }
      
      // Route to admin service or API
      if (c.env.API_SERVICE_URL) {
        const targetUrl = new URL('/admin' + path + url.search, c.env.API_SERVICE_URL);
        const fetchOptions: RequestInit = {
          method: c.req.method,
          headers: c.req.header(),
        };
        
        if (c.req.method !== 'GET' && c.req.method !== 'HEAD') {
          fetchOptions.body = await c.req.arrayBuffer();
        }
        
        return fetch(targetUrl.toString(), fetchOptions);
      }
      
      // Fallback: basic admin panel
      return c.html(`
        <!DOCTYPE html>
        <html>
        <head>
          <title>ArbEdge Admin Panel</title>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
        </head>
        <body>
          <h1>ArbEdge Admin Panel</h1>
          <p>Admin functionality placeholder</p>
          <ul>
            <li><a href="/admin/users">User Management</a></li>
            <li><a href="/admin/trading">Trading Settings</a></li>
            <li><a href="/admin/analytics">Analytics</a></li>
            <li><a href="/api/health">System Health</a></li>
          </ul>
        </body>
        </html>
      `);
    });
    
    return adminApp;
  }
  
  /**
   * Get content type based on file extension
   */
  private getContentType(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase();
    
    const contentTypes: Record<string, string> = {
      'html': 'text/html',
      'css': 'text/css',
      'js': 'application/javascript',
      'json': 'application/json',
      'png': 'image/png',
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'gif': 'image/gif',
      'svg': 'image/svg+xml',
      'ico': 'image/x-icon',
      'woff': 'font/woff',
      'woff2': 'font/woff2',
      'ttf': 'font/ttf',
      'eot': 'application/vnd.ms-fontobject',
    };
    
    return contentTypes[ext || ''] || 'application/octet-stream';
  }
  
  /**
   * Validate admin authentication
   */
  private isValidAdminAuth(authHeader: string): boolean {
    // Basic validation - in production, this should be more robust
    const token = authHeader.replace('Bearer ', '');
    
    // For now, just check if it's a non-empty token
    // In production, validate against a proper auth service
    return token.length > 10;
  }
}

// Create and export router instance
const serviceRouter = new ServiceRouter();

// Export the main API router
export const router = new Hono<{ Bindings: Env }>();

// Mount the telegram bot handler
router.route('/telegram', serviceRouter.telegramBotHandler());

// Mount the API handler
router.route('/api', serviceRouter.apiHandler());

// Mount the admin handler
router.route('/admin', serviceRouter.adminHandler());

// Mount the discord handler
router.route('/discord', serviceRouter.discordBotHandler());

// Health check endpoint
router.get('/health', (c) => {
  return c.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    service: 'celebrum-ai-worker'
  });
});