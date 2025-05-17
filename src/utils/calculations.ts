/**
 * Calculates the Annual Percentage Rate (APR) from a periodic rate.
 *
 * @param periodicRate The rate for a single period (e.g., daily, weekly).
 * @param periodsPerYear The number of periods in a year.
 * @returns The APR as a decimal (e.g., 0.05 for 5%).
 */
export function calculateAPR(periodicRate: number, periodsPerYear: number): number {
  if (periodsPerYear <= 0) {
    throw new Error('Periods per year must be greater than 0.');
  }
  // Simple APR calculation: periodicRate * periodsPerYear
  // This does not compound. For APY (compounded), the formula would be (1 + periodicRate)^periodsPerYear - 1
  return periodicRate * periodsPerYear;
}

/**
 * Calculates the Annual Percentage Rate (APR) for a given funding rate.
 * Funding rates are typically quoted per period (e.g., per 8 hours).
 *
 * @param fundingRate The funding rate per period (as a decimal, e.g., 0.0001 for 0.01%).
 * @param fundingIntervalHours The interval in hours at which the funding rate is applied (e.g., 8 for every 8 hours).
 * @returns The APR as a decimal.
 */
export function calculateFundingRateAPR(fundingRate: number, fundingIntervalHours = 8): number {
   if (fundingIntervalHours <= 0) {
     throw new Error('Funding interval hours must be greater than 0.');
   }
  const DAYS_PER_YEAR = 365;
   const periodsPerDay = 24 / fundingIntervalHours;
  const periodsPerYear = periodsPerDay * DAYS_PER_YEAR;
   return calculateAPR(fundingRate, periodsPerYear);
 }
