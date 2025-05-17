/// <reference types="vitest/globals" />
import { describe, it, expect } from 'vitest';
import {
  calculateAPR,
  calculateFundingRateAPR,
} from '../../src/utils/calculations';

describe('calculateAPR', () => {
  it('should calculate simple APR correctly', () => {
    expect(calculateAPR(0.01, 12)).toBeCloseTo(0.12); // 1% monthly rate, 12 periods
    expect(calculateAPR(0.001, 365)).toBeCloseTo(0.365); // 0.1% daily rate, 365 periods
  });

  it('should throw error for non-positive periodsPerYear', () => {
    expect(() => calculateAPR(0.01, 0)).toThrow('Periods per year must be greater than 0.');
    expect(() => calculateAPR(0.01, -1)).toThrow('Periods per year must be greater than 0.');
  });
});

describe('calculateFundingRateAPR', () => {
  it('should calculate funding rate APR correctly for 8-hour intervals', () => {
    // 0.01% funding rate every 8 hours
    // Periods per day = 24 / 8 = 3
    // Periods per year = 3 * 365 = 1095
    // APR = 0.0001 * 1095 = 0.1095 (10.95%)
    expect(calculateFundingRateAPR(0.0001, 8)).toBeCloseTo(0.1095);
  });

  it('should calculate funding rate APR correctly for 1-hour intervals', () => {
    // 0.005% funding rate every 1 hour
    // Periods per day = 24 / 1 = 24
    // Periods per year = 24 * 365 = 8760
    // APR = 0.00005 * 8760 = 0.438 (43.8%)
    expect(calculateFundingRateAPR(0.00005, 1)).toBeCloseTo(0.438);
  });

  it('should use default 8-hour interval if not provided', () => {
    expect(calculateFundingRateAPR(0.0001)).toBeCloseTo(calculateFundingRateAPR(0.0001, 8));
  });

  it('should throw error for non-positive fundingIntervalHours', () => {
    expect(() => calculateFundingRateAPR(0.0001, 0)).toThrow('Funding interval hours must be greater than 0.');
    expect(() => calculateFundingRateAPR(0.0001, -4)).toThrow('Funding interval hours must be greater than 0.');
  });
});
