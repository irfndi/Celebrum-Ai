import type { Position, TradingPairSymbol, PositionSide } from '../../src/types'; // PositionSide is correctly exported now

export const MOCK_POSITION_DEFAULTS: Position = {
  symbol: 'BTC/USDT' as TradingPairSymbol,
  side: 'long' as PositionSide,
  entryPrice: 30000,
  markPrice: 30050,
  amount: 1, // 'amount' is optional in Position type, providing a default here
  contracts: 1, // 'contracts' is optional in Position type, providing a default here
  margin: 3000,
  pnl: 50,
  leverage: 10,
  info: { detail: 'mock_position_default_info' },
};

export const mockPositionFactory = (
  options: Partial<Position> = {},
): Position => {
  const defaults: Position = {
    ...MOCK_POSITION_DEFAULTS,
    // Overwrite info with a deep copy if present in options, otherwise use default's info
    info: options.info ? { ...options.info } : { ...MOCK_POSITION_DEFAULTS.info },
  };

  const position: Position = {
    ...defaults,
    ...options,
    // Ensure info is merged if options.info exists, otherwise it will use the cloned default info
    info: options.info ? { ...defaults.info, ...options.info } : defaults.info,
  };

  return position;
};
