export declare const ENV: {
    readonly NODE_ENV: any;
    readonly IS_PRODUCTION: boolean;
    readonly IS_DEVELOPMENT: boolean;
    readonly IS_TEST: boolean;
};
export declare const API: {
    readonly DEFAULT_TIMEOUT: 30000;
    readonly MAX_RETRIES: 3;
    readonly RETRY_DELAY: 1000;
    readonly BACKOFF_MULTIPLIER: 2;
    readonly MAX_RETRY_DELAY: 10000;
    readonly RATE_LIMIT: {
        readonly WINDOW_MS: number;
        readonly MAX_REQUESTS: 100;
        readonly FREE_TIER_LIMIT: 50;
        readonly PRO_TIER_LIMIT: 200;
        readonly ULTRA_TIER_LIMIT: 500;
    };
    readonly CORS: {
        readonly ALLOWED_ORIGINS: readonly ["http://localhost:3000", "https://arbedge.com"];
        readonly ALLOWED_METHODS: readonly ["GET", "POST", "PUT", "DELETE", "OPTIONS"];
        readonly ALLOWED_HEADERS: readonly ["Content-Type", "Authorization", "X-API-Key"];
    };
};
export declare const DATABASE: {
    readonly CONNECTION_TIMEOUT: 10000;
    readonly QUERY_TIMEOUT: 30000;
    readonly MAX_CONNECTIONS: 10;
    readonly IDLE_TIMEOUT: 300000;
    readonly RETRY_ATTEMPTS: 3;
    readonly BATCH_SIZE: 1000;
};
export declare const TELEGRAM: {
    readonly MAX_MESSAGE_LENGTH: 4096;
    readonly PARSE_MODE: "HTML";
    readonly DISABLE_WEB_PAGE_PREVIEW: true;
    readonly COMMAND_TIMEOUT: 30000;
    readonly WEBHOOK_TIMEOUT: 10000;
    readonly MAX_CONNECTIONS: 40;
    readonly ALLOWED_UPDATES: readonly ["message", "callback_query", "inline_query"];
    readonly RATE_LIMIT: {
        readonly MESSAGES_PER_SECOND: 30;
        readonly MESSAGES_PER_MINUTE: 20;
        readonly MESSAGES_PER_CHAT_PER_SECOND: 1;
    };
    readonly COMMANDS: {
        readonly START: "/start";
        readonly HELP: "/help";
        readonly SETTINGS: "/settings";
        readonly BALANCE: "/balance";
        readonly OPPORTUNITIES: "/opportunities";
        readonly POSITIONS: "/positions";
        readonly STOP: "/stop";
    };
};
export declare const TRADING: {
    readonly MIN_PROFIT_THRESHOLD: 0.5;
    readonly MAX_POSITION_SIZE: 1000;
    readonly DEFAULT_LEVERAGE: 1;
    readonly MAX_LEVERAGE: 100;
    readonly STOP_LOSS_PERCENTAGE: 2;
    readonly TAKE_PROFIT_PERCENTAGE: 5;
    readonly MAX_CONCURRENT_TRADES: 10;
    readonly MIN_TRADE_AMOUNT: 10;
    readonly MAX_TRADE_AMOUNT: 100000;
    readonly SLIPPAGE_TOLERANCE: 0.1;
    readonly OPPORTUNITY_EXPIRY: 300000;
    readonly CONFIDENCE_THRESHOLD: 0.7;
    readonly EXCHANGES: {
        readonly BINANCE: {
            readonly MAKER_FEE: 0.001;
            readonly TAKER_FEE: 0.001;
            readonly WITHDRAWAL_FEE: 0.0005;
        };
        readonly BYBIT: {
            readonly MAKER_FEE: 0.001;
            readonly TAKER_FEE: 0.001;
            readonly WITHDRAWAL_FEE: 0.0005;
        };
        readonly OKX: {
            readonly MAKER_FEE: 0.0008;
            readonly TAKER_FEE: 0.001;
            readonly WITHDRAWAL_FEE: 0.0004;
        };
    };
    readonly RISK_LEVELS: {
        readonly LOW: {
            readonly MAX_POSITION_PERCENTAGE: 5;
            readonly STOP_LOSS_PERCENTAGE: 1;
            readonly MAX_LEVERAGE: 2;
        };
        readonly MEDIUM: {
            readonly MAX_POSITION_PERCENTAGE: 10;
            readonly STOP_LOSS_PERCENTAGE: 2;
            readonly MAX_LEVERAGE: 5;
        };
        readonly HIGH: {
            readonly MAX_POSITION_PERCENTAGE: 20;
            readonly STOP_LOSS_PERCENTAGE: 5;
            readonly MAX_LEVERAGE: 10;
        };
    };
};
export declare const CACHE: {
    readonly DEFAULT_TTL: 300;
    readonly SHORT_TTL: 60;
    readonly MEDIUM_TTL: 900;
    readonly LONG_TTL: 3600;
    readonly VERY_LONG_TTL: 86400;
    readonly KEYS: {
        readonly USER_PROFILE: "user:profile:";
        readonly USER_SETTINGS: "user:settings:";
        readonly USER_POSITIONS: "user:positions:";
        readonly ARBITRAGE_OPPORTUNITIES: "arbitrage:opportunities";
        readonly EXCHANGE_PRICES: "exchange:prices:";
        readonly TRADING_PAIRS: "trading:pairs:";
        readonly MARKET_DATA: "market:data:";
        readonly EXCHANGE_STATUS: "exchange:status:";
        readonly RATE_LIMIT: "rate:limit:";
        readonly SESSION: "session:";
    };
};
export declare const SECURITY: {
    readonly JWT: {
        readonly EXPIRES_IN: "7d";
        readonly REFRESH_EXPIRES_IN: "30d";
        readonly ALGORITHM: "HS256";
    };
    readonly API_KEY: {
        readonly LENGTH: 32;
        readonly EXPIRES_IN: "1y";
    };
    readonly PASSWORD: {
        readonly MIN_LENGTH: 8;
        readonly REQUIRE_UPPERCASE: true;
        readonly REQUIRE_LOWERCASE: true;
        readonly REQUIRE_NUMBERS: true;
        readonly REQUIRE_SYMBOLS: false;
    };
    readonly ENCRYPTION: {
        readonly ALGORITHM: "aes-256-gcm";
        readonly KEY_LENGTH: 32;
        readonly IV_LENGTH: 16;
    };
};
export declare const MONITORING: {
    readonly HEALTH_CHECK_INTERVAL: 30000;
    readonly METRICS_COLLECTION_INTERVAL: 60000;
    readonly LOG_LEVELS: {
        readonly ERROR: 0;
        readonly WARN: 1;
        readonly INFO: 2;
        readonly DEBUG: 3;
    };
    readonly ALERTS: {
        readonly ERROR_THRESHOLD: 10;
        readonly LATENCY_THRESHOLD: 5000;
        readonly MEMORY_THRESHOLD: 0.9;
        readonly CPU_THRESHOLD: 0.8;
    };
};
export declare const FEATURES: {
    readonly ARBITRAGE_TRADING: true;
    readonly TECHNICAL_ANALYSIS: true;
    readonly SOCIAL_TRADING: false;
    readonly ADVANCED_CHARTS: true;
    readonly MOBILE_NOTIFICATIONS: true;
    readonly EMAIL_NOTIFICATIONS: true;
    readonly TWO_FACTOR_AUTH: true;
    readonly API_ACCESS: true;
    readonly BETA_FEATURES: false;
};
export declare const SUBSCRIPTION_TIERS: {
    readonly FREE: {
        readonly MAX_POSITIONS: 3;
        readonly MAX_API_CALLS: 100;
        readonly FEATURES: readonly ["basic_arbitrage", "telegram_bot"];
        readonly PRICE: 0;
    };
    readonly PRO: {
        readonly MAX_POSITIONS: 10;
        readonly MAX_API_CALLS: 1000;
        readonly FEATURES: readonly ["advanced_arbitrage", "technical_analysis", "alerts"];
        readonly PRICE: 29.99;
    };
    readonly ULTRA: {
        readonly MAX_POSITIONS: 50;
        readonly MAX_API_CALLS: 10000;
        readonly FEATURES: readonly ["all_features", "priority_support", "custom_strategies"];
        readonly PRICE: 99.99;
    };
};
export declare const ERROR_MESSAGES: {
    readonly INVALID_INPUT: "Invalid input provided";
    readonly UNAUTHORIZED: "Unauthorized access";
    readonly NOT_FOUND: "Resource not found";
    readonly INTERNAL_ERROR: "Internal server error";
    readonly RATE_LIMITED: "Rate limit exceeded";
    readonly VALIDATION_FAILED: "Validation failed";
    readonly INSUFFICIENT_BALANCE: "Insufficient account balance";
    readonly POSITION_LIMIT_EXCEEDED: "Position limit exceeded";
    readonly INVALID_SYMBOL: "Invalid trading symbol";
    readonly EXCHANGE_ERROR: "Exchange API error";
    readonly NETWORK_ERROR: "Network connection error";
    readonly TIMEOUT_ERROR: "Request timeout";
};
export declare const SUCCESS_MESSAGES: {
    readonly CREATED: "Resource created successfully";
    readonly UPDATED: "Resource updated successfully";
    readonly DELETED: "Resource deleted successfully";
    readonly RETRIEVED: "Resource retrieved successfully";
    readonly POSITION_OPENED: "Position opened successfully";
    readonly POSITION_CLOSED: "Position closed successfully";
    readonly ORDER_PLACED: "Order placed successfully";
    readonly ORDER_CANCELLED: "Order cancelled successfully";
};
export declare const VALIDATION_RULES: {
    readonly USERNAME: {
        readonly MIN_LENGTH: 3;
        readonly MAX_LENGTH: 32;
        readonly PATTERN: RegExp;
    };
    readonly TELEGRAM_ID: {
        readonly MIN: 1;
        readonly MAX: 9999999999;
    };
    readonly SYMBOL: {
        readonly MIN_LENGTH: 3;
        readonly MAX_LENGTH: 20;
        readonly PATTERN: RegExp;
    };
    readonly AMOUNT: {
        readonly MIN: 1e-8;
        readonly MAX: 1000000000;
    };
    readonly PERCENTAGE: {
        readonly MIN: 0;
        readonly MAX: 100;
    };
};
export declare const ENDPOINTS: {
    readonly TELEGRAM_API: "https://api.telegram.org/bot";
    readonly BINANCE_API: "https://api.binance.com/api/v3";
    readonly BYBIT_API: "https://api.bybit.com/v5";
    readonly OKX_API: "https://www.okx.com/api/v5";
    readonly BITGET_API: "https://api.bitget.com/api/v2";
    readonly KUCOIN_API: "https://api.kucoin.com/api/v1";
    readonly GATE_API: "https://api.gateio.ws/api/v4";
    readonly MEXC_API: "https://api.mexc.com/api/v3";
    readonly HUOBI_API: "https://api.huobi.pro/v1";
    readonly KRAKEN_API: "https://api.kraken.com/0/public";
    readonly COINBASE_API: "https://api.exchange.coinbase.com";
};
export declare const ERROR_CODES: {
    readonly INTERNAL_ERROR: "INTERNAL_ERROR";
    readonly INVALID_REQUEST: "INVALID_REQUEST";
    readonly UNAUTHORIZED: "UNAUTHORIZED";
    readonly FORBIDDEN: "FORBIDDEN";
    readonly NOT_FOUND: "NOT_FOUND";
    readonly RATE_LIMITED: "RATE_LIMITED";
    readonly USER_NOT_FOUND: "USER_NOT_FOUND";
    readonly USER_ALREADY_EXISTS: "USER_ALREADY_EXISTS";
    readonly INVALID_CREDENTIALS: "INVALID_CREDENTIALS";
    readonly INSUFFICIENT_BALANCE: "INSUFFICIENT_BALANCE";
    readonly INVALID_SYMBOL: "INVALID_SYMBOL";
    readonly MARKET_CLOSED: "MARKET_CLOSED";
    readonly POSITION_NOT_FOUND: "POSITION_NOT_FOUND";
    readonly ORDER_FAILED: "ORDER_FAILED";
    readonly SERVICE_UNAVAILABLE: "SERVICE_UNAVAILABLE";
    readonly DATABASE_ERROR: "DATABASE_ERROR";
    readonly CACHE_ERROR: "CACHE_ERROR";
    readonly EXTERNAL_API_ERROR: "EXTERNAL_API_ERROR";
    readonly EXCHANGE_ERROR: "EXCHANGE_ERROR";
};
export declare const HTTP_STATUS: {
    readonly OK: 200;
    readonly CREATED: 201;
    readonly NO_CONTENT: 204;
    readonly BAD_REQUEST: 400;
    readonly UNAUTHORIZED: 401;
    readonly FORBIDDEN: 403;
    readonly NOT_FOUND: 404;
    readonly CONFLICT: 409;
    readonly UNPROCESSABLE_ENTITY: 422;
    readonly TOO_MANY_REQUESTS: 429;
    readonly INTERNAL_SERVER_ERROR: 500;
    readonly SERVICE_UNAVAILABLE: 503;
};
export declare function validateConfig(): boolean;
export declare function getConfigForEnvironment(): {
    ENV: {
        readonly NODE_ENV: any;
        readonly IS_PRODUCTION: boolean;
        readonly IS_DEVELOPMENT: boolean;
        readonly IS_TEST: boolean;
    };
    API: {
        readonly DEFAULT_TIMEOUT: 30000;
        readonly MAX_RETRIES: 3;
        readonly RETRY_DELAY: 1000;
        readonly BACKOFF_MULTIPLIER: 2;
        readonly MAX_RETRY_DELAY: 10000;
        readonly RATE_LIMIT: {
            readonly WINDOW_MS: number;
            readonly MAX_REQUESTS: 100;
            readonly FREE_TIER_LIMIT: 50;
            readonly PRO_TIER_LIMIT: 200;
            readonly ULTRA_TIER_LIMIT: 500;
        };
        readonly CORS: {
            readonly ALLOWED_ORIGINS: readonly ["http://localhost:3000", "https://arbedge.com"];
            readonly ALLOWED_METHODS: readonly ["GET", "POST", "PUT", "DELETE", "OPTIONS"];
            readonly ALLOWED_HEADERS: readonly ["Content-Type", "Authorization", "X-API-Key"];
        };
    };
    DATABASE: {
        readonly CONNECTION_TIMEOUT: 10000;
        readonly QUERY_TIMEOUT: 30000;
        readonly MAX_CONNECTIONS: 10;
        readonly IDLE_TIMEOUT: 300000;
        readonly RETRY_ATTEMPTS: 3;
        readonly BATCH_SIZE: 1000;
    };
    TELEGRAM: {
        readonly MAX_MESSAGE_LENGTH: 4096;
        readonly PARSE_MODE: "HTML";
        readonly DISABLE_WEB_PAGE_PREVIEW: true;
        readonly COMMAND_TIMEOUT: 30000;
        readonly WEBHOOK_TIMEOUT: 10000;
        readonly MAX_CONNECTIONS: 40;
        readonly ALLOWED_UPDATES: readonly ["message", "callback_query", "inline_query"];
        readonly RATE_LIMIT: {
            readonly MESSAGES_PER_SECOND: 30;
            readonly MESSAGES_PER_MINUTE: 20;
            readonly MESSAGES_PER_CHAT_PER_SECOND: 1;
        };
        readonly COMMANDS: {
            readonly START: "/start";
            readonly HELP: "/help";
            readonly SETTINGS: "/settings";
            readonly BALANCE: "/balance";
            readonly OPPORTUNITIES: "/opportunities";
            readonly POSITIONS: "/positions";
            readonly STOP: "/stop";
        };
    };
    TRADING: {
        readonly MIN_PROFIT_THRESHOLD: 0.5;
        readonly MAX_POSITION_SIZE: 1000;
        readonly DEFAULT_LEVERAGE: 1;
        readonly MAX_LEVERAGE: 100;
        readonly STOP_LOSS_PERCENTAGE: 2;
        readonly TAKE_PROFIT_PERCENTAGE: 5;
        readonly MAX_CONCURRENT_TRADES: 10;
        readonly MIN_TRADE_AMOUNT: 10;
        readonly MAX_TRADE_AMOUNT: 100000;
        readonly SLIPPAGE_TOLERANCE: 0.1;
        readonly OPPORTUNITY_EXPIRY: 300000;
        readonly CONFIDENCE_THRESHOLD: 0.7;
        readonly EXCHANGES: {
            readonly BINANCE: {
                readonly MAKER_FEE: 0.001;
                readonly TAKER_FEE: 0.001;
                readonly WITHDRAWAL_FEE: 0.0005;
            };
            readonly BYBIT: {
                readonly MAKER_FEE: 0.001;
                readonly TAKER_FEE: 0.001;
                readonly WITHDRAWAL_FEE: 0.0005;
            };
            readonly OKX: {
                readonly MAKER_FEE: 0.0008;
                readonly TAKER_FEE: 0.001;
                readonly WITHDRAWAL_FEE: 0.0004;
            };
        };
        readonly RISK_LEVELS: {
            readonly LOW: {
                readonly MAX_POSITION_PERCENTAGE: 5;
                readonly STOP_LOSS_PERCENTAGE: 1;
                readonly MAX_LEVERAGE: 2;
            };
            readonly MEDIUM: {
                readonly MAX_POSITION_PERCENTAGE: 10;
                readonly STOP_LOSS_PERCENTAGE: 2;
                readonly MAX_LEVERAGE: 5;
            };
            readonly HIGH: {
                readonly MAX_POSITION_PERCENTAGE: 20;
                readonly STOP_LOSS_PERCENTAGE: 5;
                readonly MAX_LEVERAGE: 10;
            };
        };
    };
    CACHE: {
        readonly DEFAULT_TTL: 300;
        readonly SHORT_TTL: 60;
        readonly MEDIUM_TTL: 900;
        readonly LONG_TTL: 3600;
        readonly VERY_LONG_TTL: 86400;
        readonly KEYS: {
            readonly USER_PROFILE: "user:profile:";
            readonly USER_SETTINGS: "user:settings:";
            readonly USER_POSITIONS: "user:positions:";
            readonly ARBITRAGE_OPPORTUNITIES: "arbitrage:opportunities";
            readonly EXCHANGE_PRICES: "exchange:prices:";
            readonly TRADING_PAIRS: "trading:pairs:";
            readonly MARKET_DATA: "market:data:";
            readonly EXCHANGE_STATUS: "exchange:status:";
            readonly RATE_LIMIT: "rate:limit:";
            readonly SESSION: "session:";
        };
    };
    SECURITY: {
        readonly JWT: {
            readonly EXPIRES_IN: "7d";
            readonly REFRESH_EXPIRES_IN: "30d";
            readonly ALGORITHM: "HS256";
        };
        readonly API_KEY: {
            readonly LENGTH: 32;
            readonly EXPIRES_IN: "1y";
        };
        readonly PASSWORD: {
            readonly MIN_LENGTH: 8;
            readonly REQUIRE_UPPERCASE: true;
            readonly REQUIRE_LOWERCASE: true;
            readonly REQUIRE_NUMBERS: true;
            readonly REQUIRE_SYMBOLS: false;
        };
        readonly ENCRYPTION: {
            readonly ALGORITHM: "aes-256-gcm";
            readonly KEY_LENGTH: 32;
            readonly IV_LENGTH: 16;
        };
    };
    MONITORING: {
        readonly HEALTH_CHECK_INTERVAL: 30000;
        readonly METRICS_COLLECTION_INTERVAL: 60000;
        readonly LOG_LEVELS: {
            readonly ERROR: 0;
            readonly WARN: 1;
            readonly INFO: 2;
            readonly DEBUG: 3;
        };
        readonly ALERTS: {
            readonly ERROR_THRESHOLD: 10;
            readonly LATENCY_THRESHOLD: 5000;
            readonly MEMORY_THRESHOLD: 0.9;
            readonly CPU_THRESHOLD: 0.8;
        };
    };
    FEATURES: {
        readonly ARBITRAGE_TRADING: true;
        readonly TECHNICAL_ANALYSIS: true;
        readonly SOCIAL_TRADING: false;
        readonly ADVANCED_CHARTS: true;
        readonly MOBILE_NOTIFICATIONS: true;
        readonly EMAIL_NOTIFICATIONS: true;
        readonly TWO_FACTOR_AUTH: true;
        readonly API_ACCESS: true;
        readonly BETA_FEATURES: false;
    } | {
        BETA_FEATURES: boolean;
        ARBITRAGE_TRADING: true;
        TECHNICAL_ANALYSIS: true;
        SOCIAL_TRADING: false;
        ADVANCED_CHARTS: true;
        MOBILE_NOTIFICATIONS: true;
        EMAIL_NOTIFICATIONS: true;
        TWO_FACTOR_AUTH: true;
        API_ACCESS: true;
    };
};
