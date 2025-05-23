# ArbEdge - Rust Implementation

A high-performance arbitrage detection and trading system built with Rust and deployed on Cloudflare Workers.

## 🚀 Features

- **Real-time Arbitrage Detection**: Monitor funding rate differences across multiple exchanges
- **Multi-Exchange Support**: Binance, Bybit, OKX, and Bitget integration
- **Automated Notifications**: Telegram bot integration for opportunity alerts
- **Position Management**: Track and manage arbitrage positions
- **High Performance**: Built with Rust for maximum speed and efficiency
- **Serverless Deployment**: Runs on Cloudflare Workers for global edge computing

## 🏗️ Architecture

This project is built using:

- **Rust**: Core application logic for performance and safety
- **Cloudflare Workers**: Serverless edge computing platform
- **KV Storage**: Persistent data storage for positions and configurations
- **Telegram API**: Real-time notifications and bot commands
- **Exchange APIs**: Direct integration with cryptocurrency exchanges

## 📋 Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (v18 or later)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/)
- Cloudflare account with Workers enabled
- Telegram Bot Token (optional, for notifications)

## 🛠️ Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/arb-edge.git
   cd arb-edge
   ```

2. **Install Rust dependencies**:
   ```bash
   cargo build
   ```

3. **Install Wrangler**:
   ```bash
   npm install -g wrangler@latest
   ```

4. **Add WASM target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## ⚙️ Configuration

### Environment Variables

Configure the following environment variables in your Cloudflare Workers dashboard or `wrangler.toml`:

```toml
[env.production.vars]
EXCHANGES = "binance,bybit,okx,bitget"
ARBITRAGE_THRESHOLD = "0.001"
MONITORED_PAIRS_CONFIG = '[{"symbol":"BTCUSDT","base":"BTC","quote":"USDT","exchange_id":"binance"}]'
TELEGRAM_BOT_TOKEN = "your_telegram_bot_token"
TELEGRAM_CHAT_ID = "your_chat_id"
```

### Exchange Configuration

The system supports multiple exchanges. Configure API credentials through the `/exchange/credentials` endpoint or directly in KV storage.

## 🚀 Deployment

### Local Development

1. **Run tests**:
   ```bash
   cargo test
   ```

2. **Check formatting**:
   ```bash
   cargo fmt --check
   ```

3. **Run linter**:
   ```bash
   cargo clippy
   ```

4. **Local development server**:
   ```bash
   wrangler dev
   ```

### Production Deployment

1. **Build for production**:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Deploy to Cloudflare Workers**:
   ```bash
   wrangler deploy
   ```

## 📡 API Endpoints

### Health Check
- `GET /health` - Service health status

### Exchange Data
- `GET /exchange/markets?exchange=binance` - Get available trading pairs
- `GET /exchange/ticker?exchange=binance&symbol=BTCUSDT` - Get ticker data
- `GET /exchange/funding?exchange=binance&symbol=BTCUSDT` - Get funding rates

### Arbitrage Opportunities
- `POST /find-opportunities` - Find current arbitrage opportunities
  ```json
  {
    "trading_pairs": ["BTCUSDT", "ETHUSDT"],
    "min_threshold": 0.01
  }
  ```

### Position Management
- `POST /positions` - Create a new position
- `GET /positions` - Get all positions
- `GET /positions/{id}` - Get specific position
- `PUT /positions/{id}` - Update position
- `DELETE /positions/{id}` - Close position

### Telegram Integration
- `POST /webhook` - Telegram webhook endpoint

## 🤖 Telegram Bot Commands

- `/start` - Initialize bot and show welcome message
- `/help` - Show available commands
- `/status` - Check bot operational status
- `/opportunities` - Show recent opportunities
- `/settings` - View current configuration

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose

# Run specific test module
cargo test integration_tests
```

### Test Categories

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end workflow testing
- **API Tests**: HTTP endpoint validation
- **Data Structure Tests**: Type safety and serialization

## 📊 Monitoring

### Scheduled Tasks

The system runs automated monitoring every minute:
- Scans for arbitrage opportunities
- Sends notifications for profitable trades
- Updates position statuses

### Logging

Structured logging is available through the built-in logger:
- Error tracking and debugging
- Performance metrics
- API request/response logging

## 🔒 Security

- **Input Validation**: All API inputs are validated
- **Rate Limiting**: Built-in protection against abuse
- **Secure Storage**: Encrypted credential storage in KV
- **CodeQL Analysis**: Automated security scanning in CI/CD

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `cargo test`
5. Check formatting: `cargo fmt --check`
6. Run linter: `cargo clippy`
7. Commit your changes: `git commit -m 'Add amazing feature'`
8. Push to the branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: Check the `/Docs` directory for detailed guides
- **Issues**: Report bugs and feature requests on GitHub Issues
- **Discussions**: Join community discussions on GitHub Discussions

## 🔄 Migration from TypeScript

This project was migrated from TypeScript to Rust for improved performance and type safety. The migration included:

- ✅ Complete rewrite of core logic in Rust
- ✅ Maintained API compatibility
- ✅ Enhanced error handling and type safety
- ✅ Improved performance and memory efficiency
- ✅ Comprehensive test coverage
- ✅ Updated CI/CD pipeline

## 📈 Performance

Rust implementation benefits:
- **Memory Safety**: Zero-cost abstractions and memory safety
- **Performance**: Significantly faster execution compared to TypeScript
- **Concurrency**: Efficient async/await with Tokio runtime
- **Type Safety**: Compile-time error detection
- **Small Binary Size**: Optimized WASM output for edge deployment

---

**Built with ❤️ using Rust and Cloudflare Workers** 