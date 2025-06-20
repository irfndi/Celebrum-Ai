export declare const APP_NAME = "ArbEdge";
export declare const APP_VERSION = "1.0.0";
export declare const APP_DESCRIPTION = "Advanced Arbitrage Trading Platform";
export declare const TIME: {
    readonly SECOND: 1000;
    readonly MINUTE: number;
    readonly HOUR: number;
    readonly DAY: number;
    readonly WEEK: number;
    readonly MONTH: number;
    readonly YEAR: number;
};
export declare const CRYPTOCURRENCIES: {
    readonly BTC: "Bitcoin";
    readonly ETH: "Ethereum";
    readonly BNB: "Binance Coin";
    readonly ADA: "Cardano";
    readonly SOL: "Solana";
    readonly XRP: "Ripple";
    readonly DOT: "Polkadot";
    readonly DOGE: "Dogecoin";
    readonly AVAX: "Avalanche";
    readonly SHIB: "Shiba Inu";
    readonly MATIC: "Polygon";
    readonly LTC: "Litecoin";
    readonly UNI: "Uniswap";
    readonly LINK: "Chainlink";
    readonly ATOM: "Cosmos";
};
export declare const FIAT_CURRENCIES: {
    readonly USD: "US Dollar";
    readonly EUR: "Euro";
    readonly GBP: "British Pound";
    readonly JPY: "Japanese Yen";
    readonly AUD: "Australian Dollar";
    readonly CAD: "Canadian Dollar";
    readonly CHF: "Swiss Franc";
    readonly CNY: "Chinese Yuan";
    readonly KRW: "South Korean Won";
    readonly INR: "Indian Rupee";
};
export declare const POPULAR_PAIRS: readonly ["BTC/USDT", "ETH/USDT", "BNB/USDT", "ADA/USDT", "SOL/USDT", "XRP/USDT", "DOT/USDT", "DOGE/USDT", "AVAX/USDT", "MATIC/USDT"];
export declare const EXCHANGE_INFO: {
    readonly binance: {
        readonly name: "Binance";
        readonly website: "https://www.binance.com";
        readonly api: "https://api.binance.com";
        readonly fees: {
            readonly maker: 0.001;
            readonly taker: 0.001;
        };
        readonly countries: readonly ["Global"];
        readonly founded: 2017;
    };
    readonly bybit: {
        readonly name: "Bybit";
        readonly website: "https://www.bybit.com";
        readonly api: "https://api.bybit.com";
        readonly fees: {
            readonly maker: 0.001;
            readonly taker: 0.001;
        };
        readonly countries: readonly ["Global"];
        readonly founded: 2018;
    };
    readonly okx: {
        readonly name: "OKX";
        readonly website: "https://www.okx.com";
        readonly api: "https://www.okx.com/api";
        readonly fees: {
            readonly maker: 0.0008;
            readonly taker: 0.001;
        };
        readonly countries: readonly ["Global"];
        readonly founded: 2017;
    };
    readonly bitget: {
        readonly name: "Bitget";
        readonly website: "https://www.bitget.com";
        readonly api: "https://api.bitget.com";
        readonly fees: {
            readonly maker: 0.001;
            readonly taker: 0.001;
        };
        readonly countries: readonly ["Global"];
        readonly founded: 2018;
    };
    readonly kucoin: {
        readonly name: "KuCoin";
        readonly website: "https://www.kucoin.com";
        readonly api: "https://api.kucoin.com";
        readonly fees: {
            readonly maker: 0.001;
            readonly taker: 0.001;
        };
        readonly countries: readonly ["Global"];
        readonly founded: 2017;
    };
};
export declare const NOTIFICATION_TYPES: {
    readonly OPPORTUNITY_FOUND: "opportunity_found";
    readonly POSITION_OPENED: "position_opened";
    readonly POSITION_CLOSED: "position_closed";
    readonly PROFIT_TARGET_HIT: "profit_target_hit";
    readonly STOP_LOSS_HIT: "stop_loss_hit";
    readonly BALANCE_LOW: "balance_low";
    readonly SYSTEM_ALERT: "system_alert";
    readonly MAINTENANCE: "maintenance";
};
export declare const WS_EVENTS: {
    readonly CONNECT: "connect";
    readonly DISCONNECT: "disconnect";
    readonly ERROR: "error";
    readonly PRICE_UPDATE: "price_update";
    readonly OPPORTUNITY_UPDATE: "opportunity_update";
    readonly POSITION_UPDATE: "position_update";
    readonly BALANCE_UPDATE: "balance_update";
    readonly NOTIFICATION: "notification";
    readonly HEARTBEAT: "heartbeat";
};
export declare const RATE_LIMITS: {
    readonly FREE_TIER: 100;
    readonly PRO_TIER: 500;
    readonly ULTRA_TIER: 2000;
    readonly ADMIN: 10000;
};
export declare const UPLOAD_LIMITS: {
    readonly MAX_FILE_SIZE: number;
    readonly ALLOWED_TYPES: readonly ["image/jpeg", "image/png", "image/gif", "application/pdf"];
    readonly MAX_FILES: 5;
};
export declare const PAGINATION: {
    readonly DEFAULT_PAGE: 1;
    readonly DEFAULT_LIMIT: 20;
    readonly MAX_LIMIT: 100;
};
export declare const REGEX: {
    readonly EMAIL: RegExp;
    readonly PHONE: RegExp;
    readonly USERNAME: RegExp;
    readonly PASSWORD: RegExp;
    readonly CRYPTO_ADDRESS: RegExp;
    readonly SYMBOL: RegExp;
    readonly TELEGRAM_USERNAME: RegExp;
};
export declare const COLORS: {
    readonly SUCCESS: "#10B981";
    readonly ERROR: "#EF4444";
    readonly WARNING: "#F59E0B";
    readonly INFO: "#3B82F6";
    readonly PRIMARY: "#6366F1";
    readonly SECONDARY: "#8B5CF6";
    readonly NEUTRAL: "#6B7280";
    readonly PROFIT: "#10B981";
    readonly LOSS: "#EF4444";
};
export declare const TIMEFRAMES: {
    readonly '1m': "1 Minute";
    readonly '5m': "5 Minutes";
    readonly '15m': "15 Minutes";
    readonly '30m': "30 Minutes";
    readonly '1h': "1 Hour";
    readonly '4h': "4 Hours";
    readonly '1d': "1 Day";
    readonly '1w': "1 Week";
    readonly '1M': "1 Month";
};
export declare const DEFAULT_SETTINGS: {
    readonly THEME: "dark";
    readonly LANGUAGE: "en";
    readonly TIMEZONE: "UTC";
    readonly CURRENCY: "USD";
    readonly NOTIFICATIONS: true;
    readonly SOUND_ALERTS: false;
    readonly AUTO_REFRESH: true;
    readonly REFRESH_INTERVAL: 30000;
};
export declare const TIER_FEATURES: {
    readonly free: readonly ["basic_arbitrage", "telegram_bot", "price_alerts", "basic_charts"];
    readonly pro: readonly ["advanced_arbitrage", "technical_analysis", "custom_alerts", "advanced_charts", "portfolio_tracking", "api_access"];
    readonly ultra: readonly ["all_features", "priority_support", "custom_strategies", "white_label", "dedicated_server", "advanced_analytics"];
};
export type CryptocurrencyType = keyof typeof CRYPTOCURRENCIES;
export type FiatCurrencyType = keyof typeof FIAT_CURRENCIES;
export type ExchangeType = keyof typeof EXCHANGE_INFO;
export type NotificationTypeType = typeof NOTIFICATION_TYPES[keyof typeof NOTIFICATION_TYPES];
export type WSEventType = typeof WS_EVENTS[keyof typeof WS_EVENTS];
export type TimeframeType = keyof typeof TIMEFRAMES;
export type TierType = keyof typeof TIER_FEATURES;
