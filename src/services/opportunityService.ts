import type {
  ExchangeId,
  TradingPairSymbol,
  ArbitrageOpportunity,
  FundingRateInfo,
  StructuredTradingPair,
  LoggerInterface,
} from "../types.ts";
import type { TelegramService } from "./telegramService";
import type { ExchangeService } from "./exchangeService";
import { pRateLimit } from "p-ratelimit";
import * as ccxt from "ccxt";

const limit = pRateLimit({
  interval: 1000,
  rate: 10,
  concurrency: 5,
});

/**
 * Service responsible for identifying arbitrage opportunities based on funding rates.
 */
export interface OpportunityServiceConfig {
  exchangeService: ExchangeService;
  telegramService: TelegramService | null;
  logger: LoggerInterface;
  monitoredPairs: StructuredTradingPair[];
  exchanges: ExchangeId[];
  threshold: number;
}

export interface IOpportunityService {
  getConfig(): OpportunityServiceConfig;
  findOpportunities(
    exchangeIds: ExchangeId[],
    pairs: TradingPairSymbol[],
    threshold: number
  ): Promise<ArbitrageOpportunity[]>;
  monitorOpportunities(threshold: number): Promise<ArbitrageOpportunity[]>;
  processOpportunities(opportunities: ArbitrageOpportunity[]): Promise<void>;
}

export class OpportunityService implements IOpportunityService {
  private exchangeService: ExchangeService;
  private telegramService: TelegramService | null = null;
  private logger: LoggerInterface;
  private monitoredPairs: StructuredTradingPair[];
  private exchanges: ExchangeId[];
  private threshold: number;

  /**
   * Creates an instance of OpportunityService.
   * @param config An instance of OpportunityServiceConfig.
   */
  constructor(config: OpportunityServiceConfig) {
    this.exchangeService = config.exchangeService;
    this.telegramService = config.telegramService;
    this.logger = config.logger;
    this.monitoredPairs = config.monitoredPairs;
    this.exchanges = config.exchanges;
    this.threshold = config.threshold;
    this.logger.info("OpportunityService initialized", {
      monitoredPairsCount: this.monitoredPairs.length,
      exchangeCount: this.exchanges.length,
      threshold: this.threshold,
    });
  }

  public getConfig(): OpportunityServiceConfig {
    return {
      exchangeService: this.exchangeService,
      telegramService: this.telegramService,
      logger: this.logger,
      monitoredPairs: this.monitoredPairs,
      exchanges: this.exchanges,
      threshold: this.threshold,
    };
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
    this.logger.debug("Finding opportunities", {
      exchangeIds,
      pairs,
      threshold,
    });

    // Step 1 from Spec (modified): Prepare for fetching.
    // Data will be fetched and stored as per spec steps 2 & 3 later.

    const fundingRateData = new Map<
      TradingPairSymbol,
      Map<ExchangeId, FundingRateInfo | null>
    >();
    const tradingFeeData = new Map<
      TradingPairSymbol,
      Map<ExchangeId, ccxt.TradingFeeInterface | null>
    >(); // Assuming ccxt.TradingFeeInterface contains taker fee

    // Step 2 & 3 from Spec: Concurrently fetch and store data
    const fetchPromises: Promise<void>[] = [];

    for (const pair of pairs) {
      if (!fundingRateData.has(pair)) {
        fundingRateData.set(
          pair,
          new Map<ExchangeId, FundingRateInfo | null>()
        );
      }
      if (!tradingFeeData.has(pair)) {
        tradingFeeData.set(
          pair,
          new Map<ExchangeId, ccxt.TradingFeeInterface | null>()
        );
      }

      for (const exchangeId of exchangeIds) {
        // Fetch FundingRateInfo
        fetchPromises.push(
          limit(async () => {
            try {
              const rateInfo = await this.exchangeService.getFundingRate(
                exchangeId,
                pair
              );
              fundingRateData.get(pair)?.set(exchangeId, rateInfo || null);
            } catch (e) {
              this.logger.error(
                `Error fetching funding rate for ${pair} on ${exchangeId}`,
                { error: e }
              );
              fundingRateData.get(pair)?.set(exchangeId, null);
            }
          })
        );

        // Fetch TradingFeesInfo
        fetchPromises.push(
          limit(async () => {
            try {
              const feeInfo = await this.exchangeService.getTradingFees(
                exchangeId,
                pair
              );
              tradingFeeData.get(pair)?.set(exchangeId, feeInfo || null);
            } catch (e) {
              this.logger.error(
                `Error fetching trading fees for ${pair} on ${exchangeId}`,
                { error: e }
              );
              tradingFeeData.get(pair)?.set(exchangeId, null);
            }
          })
        );
      }
    }

    await Promise.all(fetchPromises);

    this.logger.debug("All funding rates and trading fees fetched.");

    const opportunities: ArbitrageOpportunity[] = [];

    // Step 4 from Spec: Identify Opportunities
    for (const pair of pairs) {
      const pairFundingRates = fundingRateData.get(pair);
      const pairTradingFees = tradingFeeData.get(pair);

      if (!pairFundingRates || !pairTradingFees) {
        this.logger.warn(`Missing data for pair ${pair}, skipping.`);
        continue;
      }

      const availableExchanges = Array.from(pairFundingRates.keys()).filter(
        (exchangeId) => pairFundingRates.get(exchangeId) !== null // Ensure rate info exists
      );

      if (availableExchanges.length < 2) {
        this.logger.debug(
          `Skipping ${pair} - not enough exchanges with funding rates.`
        );
        continue;
      }

      for (let i = 0; i < availableExchanges.length; i++) {
        for (let j = i + 1; j < availableExchanges.length; j++) {
          const exchangeA = availableExchanges[i];
          const exchangeB = availableExchanges[j];

          const fundingRateInfoA = pairFundingRates.get(exchangeA);
          const fundingRateInfoB = pairFundingRates.get(exchangeB);

          // Spec 4.b.i.3: If fundingRateInfoA or fundingRateInfoB is null, continue.
          // This is already handled by availableExchanges filter, but double check for safety.
          if (!fundingRateInfoA || !fundingRateInfoB) {
            this.logger.warn(
              `Missing funding rate info for ${pair} on ${exchangeA} or ${exchangeB} despite pre-filter. Skipping.`
            );
            continue;
          }

          const tradingFeesA = pairTradingFees.get(exchangeA);
          const tradingFeesB = pairTradingFees.get(exchangeB);

          // Spec 4.b.i.4: Determine long and short exchange
          let longExchange: ExchangeId,
            shortExchange: ExchangeId,
            longRate: number,
            shortRate: number,
            longRateTimestamp: number;

          if (fundingRateInfoA.fundingRate <= fundingRateInfoB.fundingRate) {
            longExchange = exchangeA;
            shortExchange = exchangeB;
            longRate = fundingRateInfoA.fundingRate;
            shortRate = fundingRateInfoB.fundingRate;
            longRateTimestamp = fundingRateInfoA.timestamp;
          } else {
            longExchange = exchangeB;
            shortExchange = exchangeA;
            longRate = fundingRateInfoB.fundingRate;
            shortRate = fundingRateInfoA.fundingRate;
            longRateTimestamp = fundingRateInfoB.timestamp;
          }

          // Spec 4.b.i.5: Calculate rateDifference (gross)
          // rateDifference is shortRate - longRate because longRate is smaller (or more negative)
          // To get an absolute difference if order is not guaranteed, use Math.abs(rateA - rateB)
          // Given our long/short determination, shortRate - longRate will be positive if shortRate > longRate.
          const rateDifference = shortRate - longRate;

          // If rateDifference is not positive, it's not an arbitrage in this direction.
          // (e.g. if both are negative, shortRate = -0.01, longRate = -0.05, then -0.01 - (-0.05) = 0.04)
          // if shortRate = 0.05, longRate = 0.01, then 0.05 - 0.01 = 0.04
          // if shortRate = 0.01, longRate = 0.05, then 0.01 - 0.05 = -0.04 (this case shouldn't happen due to long/short assignment)
          // The spec uses Math.abs(shortRate - longRate) for `rateDifference`
          // then netRateDifference = rateDifference - totalEstimatedFees.
          // Let's stick to the spec's definition for rateDifference more closely:
          const specRateDifference = Math.abs(
            fundingRateInfoA.fundingRate - fundingRateInfoB.fundingRate
          );

          // Spec 4.b.i.6 & 7: Determine taker fee rates.
          const feeInfoLong =
            longExchange === exchangeA ? tradingFeesA : tradingFeesB;
          const feeInfoShort =
            shortExchange === exchangeA ? tradingFeesA : tradingFeesB;

          // Stricter fee handling as per spec:
          // If fee information is missing (null/undefined) for either exchange, skip this combination.
          // Assumes no "fee-free" configuration exists yet.
          if (
            !feeInfoLong ||
            typeof feeInfoLong.taker !== "number" ||
            !feeInfoShort ||
            typeof feeInfoShort.taker !== "number"
          ) {
            this.logger.warn(
              `Skipping opportunity for ${pair} between ${longExchange} and ${shortExchange} due to missing fee information.`,
              {
                feeInfoLong,
                feeInfoShort,
              }
            );
            continue; // Skip this specific exchange pair combination
          }

          const longExchangeTakerFeeRate = feeInfoLong.taker;
          const shortExchangeTakerFeeRate = feeInfoShort.taker;

          // Spec 4.b.i.8: Calculate totalEstimatedFees
          const totalEstimatedFees =
            longExchangeTakerFeeRate + shortExchangeTakerFeeRate;

          // Spec 4.b.i.9: Calculate netRateDifference
          // Using specRateDifference (gross, always positive)
          const netRateDifference = specRateDifference - totalEstimatedFees;

          // Spec 4.b.i.10: If netRateDifference >= threshold (and positive)
          // The spec says "netRateDifference > 0 && netRateDifference >= threshold".
          // If threshold can be 0 or negative, `netRateDifference > 0` is important.
          // If threshold is always positive, `netRateDifference >= threshold` implies `> 0`.
          // Assuming threshold is a positive value representing minimum profit.
          if (netRateDifference >= threshold && shortRate > longRate) {
            const opportunity: ArbitrageOpportunity = {
              pair: pair,
              longExchange,
              shortExchange,
              longRate,
              shortRate,
              rateDifference: specRateDifference,
              longExchangeTakerFeeRate,
              shortExchangeTakerFeeRate,
              totalEstimatedFees,
              netRateDifference,
              timestamp: longRateTimestamp,
            };
            opportunities.push(opportunity);
            this.logger.info("Arbitrage opportunity found", { opportunity });
            if (this.telegramService) {
              this.telegramService
                .sendOpportunityNotification(opportunity)
                .catch((teleError) => {
                  this.logger.error("Failed to send Telegram notification", {
                    teleError,
                    // Using a unique part of the opportunity for logging if ID was removed
                    opportunityDetails: `${opportunity.pair}-${opportunity.longExchange}-${opportunity.shortExchange}-${opportunity.timestamp}`,
                  });
                });
            }
          }
        }
      }
    }

    this.logger.info(`Found ${opportunities.length} opportunities.`);
    return opportunities;
  }

  /**
   * Runs opportunity checks using configured pairs and exchanges.
   * @param threshold Minimum net rate difference to filter opportunities.
   */
  async monitorOpportunities(
    threshold: number
  ): Promise<ArbitrageOpportunity[]> {
    const exchangeIds = this.exchanges;
    const pairs = this.monitoredPairs.map((p) => p.symbol);
    const opportunities = await this.findOpportunities(
      exchangeIds,
      pairs,
      threshold
    );
    this.logger.info(
      `monitorOpportunities found ${opportunities.length} opportunities with threshold ${threshold}.`
    );
    return opportunities;
  }

  /**
   * Processes identified arbitrage opportunities.
   * For now, it logs the opportunities. This method can be expanded later.
   * @param opportunities - An array of ArbitrageOpportunity objects.
   */
  async processOpportunities(
    opportunities: ArbitrageOpportunity[]
  ): Promise<void> {
    this.logger.info(`Processing ${opportunities.length} opportunities...`);
    if (opportunities.length > 0) {
      for (const op of opportunities) {
        this.logger.log(
          "info",
          `Opportunity details: ${JSON.stringify(op, null, 2)}`
        );
      }
      // Potentially, further actions like sending consolidated reports or storing them.
    }
    // If a TelegramService is configured, it's already sending notifications
    // per opportunity within findOpportunities. This method could be used for summaries
    // or other types of processing.
    this.logger.info("Finished processing opportunities.");
  }

  // Helper methods can be added here
}
