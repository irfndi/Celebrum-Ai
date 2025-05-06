#!/usr/bin/env python3
"""
Automated Perp Yield-Arb & Directional Strategy

Workflow:
 1) Analysis:
    - Fetch funding rates & order books via CCXT
 2) Plan & Strategy:
    - Compute optimal contract size based on slippage
    - Define entry price, stop-loss (SL), take-profit (TP)
 3) Execution:
    - Place market orders on Bybit (long) and Binance (short)
    - Set SL/TP orders to manage risk

Env Variables:
 - BYBIT_API_KEY, BYBIT_API_SECRET
 - BINANCE_API_KEY, BINANCE_API_SECRET
 - USDT_AMOUNT (e.g. 1000)
 - BYBIT_LEVERAGE, BINANCE_LEVERAGE (e.g. 20)

Usage:
  $ export BYBIT_API_KEY=... BYBIT_API_SECRET=... \
           BINANCE_API_KEY=... BINANCE_API_SECRET=... \
           USDT_AMOUNT=1000 BYBIT_LEVERAGE=20 BINANCE_LEVERAGE=20
  $ python3 trade.py
"""
import os
import ccxt
import argparse

def init_exchanges():
    bybit = ccxt.bybit({
        'apiKey': os.getenv('BYBIT_API_KEY'),
        'secret': os.getenv('BYBIT_API_SECRET'),
        'enableRateLimit': True,
    })
    binance = ccxt.binance({
        'apiKey': os.getenv('BINANCE_API_KEY'),
        'secret': os.getenv('BINANCE_API_SECRET'),
        'enableRateLimit': True,
        'options': {'defaultType': 'future'},
    })
    bybit.load_markets()
    binance.load_markets()
    # enable dual-side (hedge) mode for Binance futures
    try:
        binance.fapiPrivatePostPositionSideDual({'dualSidePosition': True})
        print("Binance hedge mode enabled")
    except Exception as e:
        print(f"Warning: could not enable hedge mode: {e}")
    return bybit, binance

def analysis(symbol):
    bybit, binance = init_exchanges()
    return {
        'bybit': {
            'funding': bybit.fetch_funding_rate(symbol),
            'book': bybit.fetch_order_book(symbol, 20)
        },
        'binance': {
            'funding': binance.fetch_funding_rate(symbol),
            'book': binance.fetch_order_book(symbol, 20)
        }
    }

def calculate_contracts(book, usdt_amount, slippage=0.001):
    total_cost = 0
    contracts = 0
    for price, amount in book['asks']:
        cost = price * amount
        if total_cost + cost > usdt_amount * (1 + slippage):
            remain = usdt_amount * (1 + slippage) - total_cost
            contracts += remain / price
            break
        total_cost += cost
        contracts += amount
    return round(contracts, 2)

def plan_strategy(data, usdt_amount):
    book = data['bybit']['book']
    size = calculate_contracts(book, usdt_amount)
    entry = book['asks'][0][0]
    sl = entry * 0.99
    tp = entry * 1.01
    return {'size': size, 'entry': entry, 'sl': sl, 'tp': tp}

def execute(symbol, params, bybit_lev, binance_lev, dry_run=False):
    bybit, binance = init_exchanges()
    # set leverage for perp trading dynamically based on market limits
    leverage = bybit_lev
    m_by = bybit.market(symbol)
    max_by = m_by['limits'].get('leverage', {}).get('max', leverage)
    lev_by = min(leverage, max_by)
    print(f"Setting Bybit leverage to {lev_by} (requested {leverage}, max {max_by})")
    try:
        bybit.set_leverage(lev_by, symbol)
    except ccxt.BadRequest as e:
        if 'leverage not modified' in str(e).lower():
            print(f"Bybit leverage already at {lev_by}")
        else:
            raise
    # fetch Binance market info for leverage limits
    m_bn = binance.market(symbol)
    # handle Binance max leverage fallback if None or missing
    limit_bn = m_bn.get('limits', {}).get('leverage') or {}
    max_bn = limit_bn.get('max') or binance_lev
    # if env leverage exceeds exchange max, override to exchange max
    if max_bn < binance_lev:
        print(f"Requested Binance leverage {binance_lev} > exchange max {max_bn}, overriding to {max_bn}")
    lev_bn = min(binance_lev, max_bn)
    print(f"Setting Binance leverage to {lev_bn} (requested {binance_lev}, max {max_bn})")
    try:
        binance.set_leverage(lev_bn, symbol)
    except ccxt.BadRequest as e:
        msg = str(e).lower()
        if 'leverage not modified' in msg:
            print(f"Binance leverage already at {lev_bn}")
        elif 'not valid' in msg:
            print(f"Binance leverage {lev_bn} not valid, using max {max_bn}")
            try:
                binance.set_leverage(max_bn, symbol)
                lev_bn = max_bn
            except Exception as e2:
                print(f"Failed to set Binance max leverage {max_bn}: {e2}")
        else:
            print(f"Error setting Binance leverage: {e}")
    except Exception as e:
        print(f"Unexpected error setting Binance leverage: {e}")
    size, sl, tp = params['size'], params['sl'], params['tp']
    entry_price = params['entry']
    try:
        usdt_exec = float(os.getenv('USDT_AMOUNT', size * entry_price))
    except:
        usdt_exec = size * entry_price
    max_qty = usdt_exec * lev_bn / entry_price
    hedge_size = min(size, max_qty)
    print(f"Using Binance hedge size {hedge_size:.2f} (target={size}, max={max_qty:.2f})")
    if dry_run:
        print(f"Dry-run: would OPEN {size} {symbol} long on Bybit and SHORT on Binance")
        print(f"Dry-run: ENTRY={params['entry']} SL={sl} TP={tp}")
        print(f"Dry-run: would SET Bybit SL={sl:.6f}")
        print(f"Dry-run: would SET Bybit TP={tp:.6f}")
        print(f"Dry-run: would SET Binance SL={sl:.6f}")
        print(f"Dry-run: would SET Binance TP={tp:.6f}")
        return {'bybit_entry': None, 'binance_entry': None}
    print(f"Opening {size} contracts of {symbol} long on Bybit and short on Binance...")
    # place Bybit long order
    try:
        ob1 = bybit.create_market_buy_order(symbol, size)
    except ccxt.ExchangeError as e:
        print(f"Bybit buy order failed: {e}")
        raise
    # place Binance short order with hedge mode and positionSide
    try:
        ob2 = binance.create_order(
            symbol, 'market', 'sell', hedge_size, None,
            {'positionSide': 'SHORT'}
        )
    except ccxt.ExchangeError as e:
        print(f"Binance sell order failed: {e}")
        # attempt to close long first if needed
        try:
            positions = binance.fetch_positions()
            pos = next((p for p in positions if p.get('symbol') == symbol), None)
            current = float(pos.get('contracts', pos.get('positionAmt', 0))) if pos else 0.0
            if current > 0:
                print(f"Closing existing long of {current:.2f} contracts")
                binance.create_order(symbol, 'market', 'sell', current, None, {'positionSide': 'BOTH'})
                print(f"Retrying short of {hedge_size:.2f} contracts")
                ob2 = binance.create_order(symbol, 'market', 'sell', hedge_size, None, {'positionSide': 'SHORT'})
            else:
                ob2 = None
        except Exception as e2:
            print(f"Binance hedge retry failed: {e2}")
            ob2 = None
    print("Setting SL/TP orders...")
    # Bybit SL via STOP_MARKET
    try:
        bybit.create_order(symbol, 'STOP_MARKET', 'sell', size, None,
                            { 'stopPrice': sl, 'triggerDirection': 'below', 'closePosition': True })
        print("Bybit SL placed")
    except Exception as e:
        print(f"Bybit SL failed: {e}")
    # Bybit TP via TAKE_PROFIT_MARKET
    try:
        bybit.create_order(symbol, 'TAKE_PROFIT_MARKET', 'sell', size, None,
                            { 'stopPrice': tp, 'triggerDirection': 'above', 'closePosition': True })
        print("Bybit TP placed")
    except Exception as e:
        print(f"Bybit TP failed: {e}")
    # Binance SL/TP for short
    if ob2:
        # TP (buy) to close short at SL price
        try:
            binance.create_order(symbol, 'TAKE_PROFIT_MARKET', 'buy', hedge_size, None,
                                 { 'stopPrice': sl, 'positionSide': 'SHORT', 'closePosition': True })
            print("Binance TP placed")
        except Exception as e:
            print(f"Binance TP failed: {e}")
        # SL (buy) to close short at TP price
        try:
            binance.create_order(symbol, 'STOP_MARKET', 'buy', hedge_size, None,
                                 { 'stopPrice': tp, 'positionSide': 'SHORT', 'closePosition': True })
            print("Binance SL placed")
        except Exception as e:
            print(f"Binance SL failed: {e}")
    else:
        print("Skipping Binance SL/TP due to no short position")
    return {'bybit_entry': ob1, 'binance_entry': ob2}

def parse_args():
    parser = argparse.ArgumentParser(description='Automated perp yield arb & directional strategy')
    parser.add_argument('--symbol', default=os.getenv('SYMBOL', 'ALPACA/USDT:USDT'),
                        help='Perpetual symbol to trade, e.g. ALPACA/USDT:USDT')
    parser.add_argument('--usdt_amount', type=float, default=float(os.getenv('USDT_AMOUNT', '1000')),
                        help='USDT amount to deploy')
    parser.add_argument('--bybit_leverage', type=int, default=int(os.getenv('BYBIT_LEVERAGE', '20')),
                        help='Bybit leverage')
    parser.add_argument('--binance_leverage', type=int, default=int(os.getenv('BINANCE_LEVERAGE', '20')),
                        help='Binance leverage')
    parser.add_argument('--dry_run', action='store_true',
                        help='Simulate trades without placing orders')
    return parser.parse_args()

def main():
    args = parse_args()
    symbol = args.symbol
    usdt_amount = args.usdt_amount
    data = analysis(symbol)
    plan = plan_strategy(data, usdt_amount)
    print("Strategy Plan:", plan)
    result = execute(symbol, plan, args.bybit_leverage, args.binance_leverage, args.dry_run)
    print("Execution Result:\n", result)

if __name__ == '__main__':
    main()
