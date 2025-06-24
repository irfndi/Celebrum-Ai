// Telegram Bot API Types
export interface TelegramUpdate {
  update_id: number;
  message?: TelegramMessage;
  edited_message?: TelegramMessage;
  channel_post?: TelegramMessage;
  edited_channel_post?: TelegramMessage;
  callback_query?: TelegramCallbackQuery;
}

export interface TelegramMessage {
  message_id: number;
  from?: TelegramUser;
  chat: TelegramChat;
  date: number;
  text?: string;
  entities?: TelegramMessageEntity[];
}

export interface TelegramUser {
  id: number;
  is_bot: boolean;
  first_name: string;
  last_name?: string;
  username?: string;
  language_code?: string;
}

export interface TelegramChat {
  id: number;
  type: 'private' | 'group' | 'supergroup' | 'channel';
  title?: string;
  username?: string;
  first_name?: string;
  last_name?: string;
}

export interface TelegramMessageEntity {
  type: string;
  offset: number;
  length: number;
  url?: string;
  user?: TelegramUser;
}

export interface TelegramCallbackQuery {
  id: string;
  from: TelegramUser;
  message?: TelegramMessage;
  data?: string;
}

// Bot Response Types
export interface TelegramBotResponse {
  method: string;
  chat_id: number;
  text?: string;
  reply_markup?: TelegramInlineKeyboard;
  parse_mode?: 'HTML' | 'Markdown' | 'MarkdownV2';
}

export interface TelegramInlineKeyboard {
  inline_keyboard: TelegramInlineKeyboardButton[][];
}

export interface TelegramInlineKeyboardButton {
  text: string;
  callback_data?: string;
  url?: string;
}

// Handler Types
export interface TelegramHandler {
  command: string;
  description: string;
  handler: (update: TelegramUpdate, env: Env) => Promise<TelegramBotResponse | null>;
}

import type { Env } from '@celebrum-ai/shared';

export interface TelegramWebhookContext {
  env: Env;
  ctx: ExecutionContext;
}

// Integration Types for Main Worker
export interface TelegramIntegrationConfig {
  botToken: string;
  webhookUrl: string;
  allowedUsers?: number[];
  adminUsers?: number[];
}

export interface TelegramServiceInterface {
  processUpdate(update: TelegramUpdate, env: Env): Promise<Response>;
  sendMessage(chatId: number, text: string, options?: any): Promise<boolean>;
  setWebhook(url: string): Promise<boolean>;
  deleteWebhook(): Promise<boolean>;
}