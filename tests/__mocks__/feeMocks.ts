import type { TradingFees } from 'ccxt'; // Assuming ccxt.TradingFees is the correct type

export function MOCK_TRADING_FEES_FACTORY(): TradingFees {
  return {
  info: { mockFeeInfo: 'some_detail' },
  BTCUSDT: {
    symbol: 'BTC/USDT',
    maker: 0.001,
    taker: 0.001,
    percentage: true,
    tierBased: false,
  },
  ETHUSDT: {
    symbol: 'ETH/USDT',
    maker: 0.001,
    taker: 0.001,
    percentage: true,
    tierBased: false,
  },
  };
}
