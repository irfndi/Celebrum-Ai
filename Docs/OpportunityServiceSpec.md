# Opportunity Service Specification

## Overview
`OpportunityService` detects arbitrage opportunities based on funding rates and trading fees across multiple exchanges.

## Configuration
`OpportunityServiceConfig`:
- `exchangeService: ExchangeService` – external service for fetching rates/fees.
- `telegramService: TelegramService | null` – optional, sends notifications.
- `logger: Logger` – logs info, warnings, errors.
- `monitoredPairs: StructuredTradingPair[]` – list of trading pairs to monitor.
- `exchanges: ExchangeId[]` – list of exchange IDs to check.

## Types
### ArbitrageOpportunity
| Field                     | Type                  | Description                                                  |
| ------------------------- | --------------------- | ------------------------------------------------------------ |
| `pair`                    | `TradingPairSymbol`   | e.g. 'BTC/USDT'                                              |
| `longExchange`            | `ExchangeId`          | Exchange to open a long position (lower funding rate)        |
| `shortExchange`           | `ExchangeId`          | Exchange to open a short position (higher funding rate)      |
| `longRate`                | `number`              | Funding rate on `longExchange`                              |
| `shortRate`               | `number`              | Funding rate on `shortExchange`                             |
| `rateDifference`          | `number`              | Absolute funding rate difference                            |
| `longExchangeTakerFeeRate`| `number`              | Taker fee rate for `longExchange`                           |
| `shortExchangeTakerFeeRate`| `number`             | Taker fee rate for `shortExchange`                          |
| `totalEstimatedFees`      | `number`              | Sum of both taker fees                                       |
| `netRateDifference`       | `number`              | `rateDifference - totalEstimatedFees`                       |
| `timestamp`               | `number`              | Epoch ms when the funding rate was retrieved on the long exchange |

## Methods

### findOpportunities(
  `exchangeIds: ExchangeId[]`,
  `pairs: TradingPairSymbol[]`,
  `threshold: number` – The minimum **net** rate difference required to consider an opportunity. This is an absolute value representing the decimal form of a funding rate (e.g., `0.0005` represents a 0.05% difference, or 5 basis points). An opportunity is logged if `netRateDifference >= threshold`.
): `Promise<ArbitrageOpportunity[]>`

**Behavior:**
1. Create a list of all unique `(exchangeId, pair)` combinations from the input `exchangeIds` and `pairs`.
2. **Concurrently fetch Data:**
   a. For each `(exchangeId, pair)` combination, fetch **FundingRateInfo** (symbol, fundingRate, timestamp, datetime, info) via `exchangeService.getFundingRate(exchangeId, pair)`.
   b. For each `(exchangeId, pair)` combination, fetch **TradingFees** (containing taker fee rate) via `exchangeService.getTradingFees(exchangeId, pair)`.
   c. Use `Promise.all` to execute these fetches in parallel.
3. **Store Fetched Data:**
   a. Store `FundingRateInfo` in a nested map: `Map<TradingPairSymbol, Map<ExchangeId, FundingRateInfo | null>>`.
   b. Store `TradingFees` in a nested map: `Map<TradingPairSymbol, Map<ExchangeId, TradingFeesInfo | null>>`.
      (Note: `TradingFeesInfo` should be defined or understood to contain at least the `taker` fee rate).
4. **Identify Opportunities:**
   a. Initialize an empty array `opportunities: ArbitrageOpportunity[]`.
   b. For each `pair` in the input `pairs`:
      i. For every unique combination of two distinct exchanges (`exchangeA`, `exchangeB`) from the input `exchangeIds`:
         1. Retrieve `fundingRateInfoA` and `fundingRateInfoB` from the stored funding rate map for the current `pair` and `exchangeA`/`exchangeB`.
         2. Retrieve `tradingFeesA` and `tradingFeesB` from the stored trading fees map for the current `pair` and `exchangeA`/`exchangeB`.
         3. If `fundingRateInfoA` or `fundingRateInfoB` is `null`, continue to the next combination (cannot determine opportunity).
         4. Determine which exchange is `longExchange` (lower funding rate) and `shortExchange` (higher funding rate). Let their respective rates be `longRate` and `shortRate`.
         5. Calculate `rateDifference = Math.abs(shortRate - longRate)`. This value is always positive, representing the gross difference before fees.
         6. Determine `longExchangeTakerFeeRate`: Use taker fee from `tradingFees` of `longExchange` for the `pair`. Default to 0 if fees are unavailable or `taker` field is missing.
         7. Determine `shortExchangeTakerFeeRate`: Use taker fee from `tradingFees` of `shortExchange` for the `pair`. Default to 0 if fees are unavailable or `taker` field is missing.
         8. Calculate `totalEstimatedFees = longExchangeTakerFeeRate + shortExchangeTakerFeeRate`.
         9. Calculate `netRateDifference = rateDifference - totalEstimatedFees`. This value can be negative if fees exceed the gross rate difference.
         10. If `netRateDifference > 0 && netRateDifference >= threshold`: (A positive net difference exceeding the threshold indicates a potential opportunity)
             - Construct an `ArbitrageOpportunity` object:
               - `pair`: current `pair`
               - `longExchange`: `exchangeId` of the long side
               - `shortExchange`: `exchangeId` of the short side
               - `longRate`: funding rate on `longExchange`
               - `shortRate`: funding rate on `shortExchange`
               - `rateDifference`: calculated gross difference
               - `longExchangeTakerFeeRate`: calculated taker fee for long side
               - `shortExchangeTakerFeeRate`: calculated taker fee for short side
               - `totalEstimatedFees`: sum of taker fees
               - `netRateDifference`: calculated net difference
               - `timestamp`: use timestamp from `FundingRateInfo` of the `longExchange`.
             - Add the new opportunity to the `opportunities` array.
             - Log the found opportunity via `logger.log(...)`.
             - If `telegramService` is configured, asynchronously call `telegramService.sendOpportunityNotification(opportunity)`.
5. Return the `opportunities` array.

**Edge Cases:**
- Skip any exchange/pair if fetching `FundingRateInfo` returns `null`.
- Treat missing fees or missing `taker` fields with caution:
          - **Justification Note:** Most CEX APIs reliably return taker fees. If fee data is truly unavailable for a pair, or if the `taker` field is missing, the exchange might indeed charge 0 for specific promotions or pair types. However, assuming 0 by default can lead to overstated profits and false-positive alerts if actual fees apply.
          - **Safe Handling:**
            - Only default to a taker fee of `0` if the pair is explicitly configured as "fee-free" in the bot's upstream configuration (e.g., based on known exchange promotions or specific account tiers).
            - If fee information is missing and the pair is not marked as "fee-free", the pair should be skipped for opportunity calculation for that exchange to prevent trading based on incomplete or incorrect fee assumptions.

### monitorOpportunities(
  `threshold: number`
): `Promise<ArbitrageOpportunity[]>`

- Delegate to `findOpportunities` using configured `exchanges` and `monitoredPairs.map(p => p.symbol)`.
+ **Behavior:**
+ 1. Retrieve configured `exchanges` and `monitoredPairs`.
+ 2. Map each `StructuredTradingPair` to its `symbol`.
+ 3. Invoke `findOpportunities(exchangeIds, pairs, threshold)`.
+ 4. Log the number of opportunities via `logger.info`.
+ 5. Return the resulting `ArbitrageOpportunity[]`.

## Usage Example
```ts
const config: OpportunityServiceConfig = {
  exchangeService,
  telegramService,
  logger,
  monitoredPairs: [{ symbol: 'BTC/USDT', base: 'BTC', quote: 'USDT' }],
  exchanges: ['binance','bybit'],
};
const svc = new OpportunityService(config);
const ops = await svc.findOpportunities(['binance','bybit'], ['BTC/USDT'], 0.0005);
console.log(ops);

// Alternatively, using monitorOpportunities:
const monitoredOps = await svc.monitorOpportunities(0.0005);
console.log(monitoredOps);
