CREATE TABLE `users` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`telegram_id` text NOT NULL,
	`first_name` text,
	`last_name` text,
	`username` text,
	`language_code` text,
	`email` text,
	`role` text DEFAULT 'free' NOT NULL,
	`status` text DEFAULT 'active' NOT NULL,
	`created_at` integer DEFAULT (unixepoch()) NOT NULL,
	`updated_at` integer DEFAULT (unixepoch()) NOT NULL,
	`last_active_at` integer,
	`settings` text DEFAULT '{}',
	`api_limits` text DEFAULT '{}',
	`account_balance` text DEFAULT '0.00',
	`beta_expires_at` integer,
	`trading_preferences` text DEFAULT '{}'
);
--> statement-breakpoint
CREATE UNIQUE INDEX `users_telegram_id_unique` ON `users` (`telegram_id`);--> statement-breakpoint
CREATE UNIQUE INDEX `users_email_unique` ON `users` (`email`);--> statement-breakpoint
CREATE UNIQUE INDEX `users_username_unique` ON `users` (`username`);--> statement-breakpoint
CREATE TABLE `opportunities` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`type` text NOT NULL,
	`symbol` text NOT NULL,
	`exchange_1` text NOT NULL,
	`exchange_2` text NOT NULL,
	`price_1` real NOT NULL,
	`price_2` real NOT NULL,
	`profit_percentage` real NOT NULL,
	`confidence` real NOT NULL,
	`expires_at` integer NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`created_at` integer DEFAULT (unixepoch()) NOT NULL
);
--> statement-breakpoint
CREATE INDEX `opportunities_type_idx` ON `opportunities` (`type`);--> statement-breakpoint
CREATE INDEX `opportunities_profit_idx` ON `opportunities` (`profit_percentage`);--> statement-breakpoint
CREATE INDEX `opportunities_active_idx` ON `opportunities` (`is_active`);--> statement-breakpoint
CREATE INDEX `opportunities_expires_idx` ON `opportunities` (`expires_at`);--> statement-breakpoint
CREATE TABLE `positions` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`user_id` integer NOT NULL,
	`exchange_id` text NOT NULL,
	`symbol` text NOT NULL,
	`type` text NOT NULL,
	`strategy` text NOT NULL,
	`entry_price` real NOT NULL,
	`exit_price` real,
	`quantity` real NOT NULL,
	`leverage` real DEFAULT 1,
	`stop_loss` real,
	`take_profit` real,
	`status` text DEFAULT 'open' NOT NULL,
	`pnl` real DEFAULT 0,
	`fees` real DEFAULT 0,
	`metadata` text DEFAULT '{}',
	`created_at` integer DEFAULT (unixepoch()) NOT NULL,
	`updated_at` integer DEFAULT (unixepoch()) NOT NULL,
	`closed_at` integer,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `positions_user_id_idx` ON `positions` (`user_id`);--> statement-breakpoint
CREATE INDEX `positions_status_idx` ON `positions` (`status`);--> statement-breakpoint
CREATE INDEX `positions_symbol_idx` ON `positions` (`symbol`);--> statement-breakpoint
CREATE INDEX `positions_strategy_idx` ON `positions` (`strategy`);--> statement-breakpoint
CREATE TABLE `trading_strategies` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`user_id` integer NOT NULL,
	`name` text NOT NULL,
	`type` text NOT NULL,
	`is_active` integer DEFAULT true NOT NULL,
	`settings` text NOT NULL,
	`performance` text DEFAULT '{}',
	`created_at` integer DEFAULT (unixepoch()) NOT NULL,
	`updated_at` integer DEFAULT (unixepoch()) NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `trading_strategies_user_id_idx` ON `trading_strategies` (`user_id`);--> statement-breakpoint
CREATE INDEX `trading_strategies_type_idx` ON `trading_strategies` (`type`);--> statement-breakpoint
CREATE INDEX `trading_strategies_active_idx` ON `trading_strategies` (`is_active`);