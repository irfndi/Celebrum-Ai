import type { ExchangeId, TradingPairSymbol, ArbitrageOpportunity, FundingRateInfo, StructuredTradingPair } from '../types';
import type { ExchangeService } from './exchangeService';
import type { TelegramService } from './telegramService';
import { pRateLimit } from 'p-ratelimit';

interface Logger {
    log: (message: string) => void;
    error: (message: string, error?: unknown) => void;
    warn: (message: string) => void;
    info: (message: string) => void;
}

const limit = pRateLimit({
    interval: 1000, 
    rate: 10,       
    concurrency: 5, 
});

/**
 * Service responsible for identifying arbitrage opportunities based on funding rates.
 */
export class OpportunityService {
  private exchangeService: ExchangeService;
  private telegramService: TelegramService | null = null;
  private logger: Logger;

  /**
   * Creates an instance of OpportunityService.
   * @param exchangeService An instance of ExchangeService to fetch market data.
   * @param telegramService An instance of TelegramService to send notifications.
   * @param logger Optional logger instance (defaults to console).
   */
  constructor(
    exchangeService: ExchangeService, 
    telegramService: TelegramService | null, 
    logger: Logger = console
  ) {
    this.exchangeService = exchangeService;
    this.telegramService = telegramService;
    this.logger = logger;
    this.logger.info('OpportunityService initialized.');
  }

  /**
   * Finds funding rate arbitrage opportunities across specified exchanges and pairs.
   *
   * @param exchangeIds - An array of exchange IDs to check (e.g., ['binance', 'bybit']).
   * @param pairs - An array of trading pairs to check (e.g., ['BTC/USDT', 'ETH/USDT']).
   * @param threshold - The minimum absolute funding rate difference required to identify an opportunity.
   * @returns A promise that resolves to an array of identified ArbitrageOpportunity objects.
   */
  async findOpportunities(
    exchangeIds: ExchangeId[],
    pairs: TradingPairSymbol[],
    threshold: number
  ): Promise<ArbitrageOpportunity[]> {
    console.log(`Searching for opportunities across exchanges: ${exchangeIds.join(', ')} for pairs: ${pairs.join(', ')} with threshold: ${threshold}`);
    const opportunities: ArbitrageOpportunity[] = [];

    // Store fetched rates: Map<Pair, Map<ExchangeId, Rate | null>>
    const allRates = new Map<TradingPairSymbol, Map<ExchangeId, number | null>>();

    // Fetch all rates concurrently
    const ratePromises: Promise<void>[] = [];

    for (const pair of pairs) {
      const pairRates = new Map<ExchangeId, number | null>();
      allRates.set(pair, pairRates);
      for (const exchangeId of exchangeIds) {
        ratePromises.push(
          this.exchangeService.getFundingRate(exchangeId, pair)
            .then(rateInfo => { 
              const rate = rateInfo?.fundingRate; 
              pairRates.set(exchangeId, typeof rate === 'number' ? rate : null); 
              if (rateInfo === null) {
                // Already handled by exchangeService logging, but we can add more here if needed
                // console.warn(`Failed to fetch funding rate info for ${pair} on ${exchangeId}`);
              } else if (typeof rate !== 'number') {
                console.warn(`Invalid funding rate data for ${pair} on ${exchangeId}:`, rateInfo);
              }
            })
        );
      }
    }

    // Wait for all rate fetching to complete
    await Promise.all(ratePromises);

    // console.log('Finished fetching rates:', allRates); // Log fetched rates (can be large)

    // Compare rates for each pair across all combinations of exchanges
    for (const [pair, pairRates] of allRates.entries()) {
      const availableExchanges = Array.from(pairRates.keys());

      for (let i = 0; i < availableExchanges.length; i++) {
        for (let j = i + 1; j < availableExchanges.length; j++) {
          const exchangeA = availableExchanges[i];
          const exchangeB = availableExchanges[j];

          const rateA = pairRates.get(exchangeA);
          const rateB = pairRates.get(exchangeB);

          // Ensure both rates are valid numbers
          if (typeof rateA === 'number' && typeof rateB === 'number') {
            const difference = Math.abs(rateA - rateB);

            if (difference >= threshold) {
              // Opportunity found!
              const longExchange = rateA < rateB ? exchangeA : exchangeB;
              const shortExchange = rateA < rateB ? exchangeB : exchangeA;
              const longRate = Math.min(rateA, rateB);
              const shortRate = Math.max(rateA, rateB);

              const opportunity: ArbitrageOpportunity = {
                pair: pair,
                longExchange: longExchange,
                shortExchange: shortExchange,
                longRate: longRate,
                shortRate: shortRate,
                rateDifference: Math.abs(longRate - shortRate), // Use absolute difference
                timestamp: Date.now(),
              };
              opportunities.push(opportunity);
              this.logger.log(`Found opportunity: ${pair} on ${longExchange}(${longRate}) vs ${shortExchange}(${shortRate}), Diff: ${Math.abs(longRate - shortRate)}`);

              // Send notification via Telegram if service is available
              if (this.telegramService) {
                // Don't await this, let it send in the background
                this.telegramService.sendOpportunityNotification(opportunity).catch(err => {
                  this.logger.error(`Error sending Telegram notification for ${pair}:`, err);
                });
              }
            }
          }
        }
      }
    }

    this.logger.info(`Found ${opportunities.length} opportunities.`);
    return opportunities;
  }

  // Helper methods can be added here
}
