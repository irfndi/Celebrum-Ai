import { describe, it, expect } from 'vitest';
import {
  escapeMarkdownV2,
  formatOpportunityMessage,
} from '../../src/utils/formatter';
import type {
  ArbitrageOpportunity,
  ExchangeId,
  // OpportunityType, // Removed as ArbitrageOpportunity.type is string;
} from '../../src/types';

// Kept describe block for escapeMarkdownV2 tests
describe("escapeMarkdownV2", () => {
    it("should return 'N/A' for undefined input", () => {
      expect(escapeMarkdownV2(undefined)).toBe("N/A");
    });

    it("should not alter a string without special characters", () => {
      expect(escapeMarkdownV2("hello world")).toBe("hello world");
    });

    it("should escape a number, including decimal points", () => {
      expect(escapeMarkdownV2(123.45)).toBe("123\\.45");
    });

    it("should escape all MarkdownV2 special characters", () => {
      const specialChars = "_*[]()~`>#+-=|{}.!";
      // Programmatically generate the expected string to avoid typos or subtle errors
      const expectedEscapedChars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!']
        .map(char => `\\${char}`)
        .join('');
      expect(escapeMarkdownV2(specialChars)).toBe(expectedEscapedChars);
    });

    it("should escape MarkdownV2 special characters within a sentence", () => {
      const inputText = "Example [Link](http://example.com) with *bold* and _italic_.";
      const expectedText =
        "Example \\[Link\\]\\(http://example\\.com\\) with \\*bold\\* and \\_italic\\_\\.";
      expect(escapeMarkdownV2(inputText)).toBe(expectedText);
    });

    it("should handle an empty string input", () => {
      expect(escapeMarkdownV2("")).toBe("");
    });
});

// Kept and potentially expanded describe block for formatOpportunityMessage tests
describe("formatOpportunityMessage", () => {
    const baseOpportunity: ArbitrageOpportunity = {
      pair: "BTC/USD",
      longExchange: "ExchangeA" as ExchangeId,
      shortExchange: "ExchangeB" as ExchangeId,
      longRate: 0.0005,
      shortRate: -0.001,
      rateDifference: 0.0015,
      // Mandatory fields as per ArbitrageOpportunity type
      longExchangeTakerFeeRate: 0.001, 
      shortExchangeTakerFeeRate: 0.001,
      totalEstimatedFees: 0.002,
      netRateDifference: 0.0014,
      timestamp: 1678886400000,
      // Optional fields
      type: "fundingRate", 
      potentialProfitValue: 150.55,
      details: "Additional details about this opportunity.",
      id: "test-id-123",
      // status: "active", // Removed, not in ArbitrageOpportunity type
    };

    it("should correctly format a 'fundingRate' opportunity with all fields", () => {
      const opportunityWithDetails: ArbitrageOpportunity = {
        ...baseOpportunity,
        details: "Ensured details for this test case.", // details is optional, so this is fine
      };
      const message = formatOpportunityMessage(opportunityWithDetails);
      const expectedDate = escapeMarkdownV2(
        new Date(opportunityWithDetails.timestamp).toLocaleString()
      );

      expect(message).toContain("🚨 *Arbitrage Opportunity Detected* 🚨");
      expect(message).toContain("📈 *Pair:* `BTC/USD`");
      expect(message).toContain(
        "↔️ *Action:* LONG `EXCHANGEA` / SHORT `EXCHANGEB`"
      );
      expect(message).toContain("*Rates \\(Funding\\):*");
      expect(message).toContain("   \\- Long \\(EXCHANGEA\\): `0\\.0500%`"); 
      expect(message).toContain("   \\- Short \\(EXCHANGEB\\): `\\-0\\.1000%`"); 
      expect(message).toContain("💰 *Gross Difference:* `0\\.1500%`");
      expect(message).toContain("💹 *Net Difference:* `0\\.1400%`"); 
      expect(message).toContain(
        // potentialProfitValue is optional, its formatting is handled
        `💸 *Potential Profit:* \\~$${escapeMarkdownV2(baseOpportunity.potentialProfitValue?.toFixed(2))}`
      );
      expect(message).toContain(`🕒 *Timestamp:* ${expectedDate}`);
      if (opportunityWithDetails.details) {
        expect(message).toContain(
          `📝 *Details:* ${escapeMarkdownV2(opportunityWithDetails.details)}`
        );
      }
    });

    it("should omit net profit section if netRateDifference is effectively missing for formatter", () => {
      // Test formatter's handling of a case that type ArbitrageOpportunity makes mandatory
      // The formatter itself checks `typeof netRateDifference === "number"`
      const opportunity = { ...baseOpportunity, netRateDifference: undefined } as any; 
      const message = formatOpportunityMessage(opportunity);
      expect(message).not.toContain("💹 *Net Difference:*");
    });

    it("should omit potential profit section if potentialProfitValue is undefined", () => {
      // potentialProfitValue is optional in ArbitrageOpportunity
      const opportunity: ArbitrageOpportunity = { ...baseOpportunity, potentialProfitValue: undefined };
      const message = formatOpportunityMessage(opportunity);
      expect(message).not.toContain("💸 *Potential Profit:*");
    });

    it("should omit details section if details is undefined", () => {
      // details is optional in ArbitrageOpportunity
      const opportunity: ArbitrageOpportunity = { ...baseOpportunity, details: undefined };
      const message = formatOpportunityMessage(opportunity);
      expect(message).not.toContain("📝 *Details:*");
    });

    it("should use 'N/A%' for undefined longRate when formatter processes it", () => {
      // Test formatter's handling if longRate data were missing/invalid despite type
      // Formatter checks `typeof longRate === "number"`
      const opportunity = { ...baseOpportunity, longRate: undefined } as any;
      const message = formatOpportunityMessage(opportunity);
      expect(message).toContain("   \\- Long \\(EXCHANGEA\\): `N/A%`");
    });

    it("should use 'N/A%' for undefined shortRate when formatter processes it", () => {
      // Test formatter's handling if shortRate data were missing/invalid despite type
      const opportunity = { ...baseOpportunity, shortRate: undefined } as any;
      const message = formatOpportunityMessage(opportunity);
      expect(message).toContain("   \\- Short \\(EXCHANGEB\\): `N/A%`");
    });
    
    it("should correctly format a 'generic' type opportunity", () => {
      const genericOpp: ArbitrageOpportunity = {
        ...baseOpportunity, // Includes all mandatory fields
        type: "generic",
        longExchange: "GenEx1" as ExchangeId, // Overrides base
        shortExchange: "GenEx2" as ExchangeId, // Overrides base
        rateDifference: 0.0025, // Overrides base, used as Gross Metric for generic
      };
      const message = formatOpportunityMessage(genericOpp);
      const expectedDate = escapeMarkdownV2(
        new Date(genericOpp.timestamp).toLocaleString()
      );

      expect(message).toContain("🚨 *Arbitrage Opportunity Detected* 🚨");
      expect(message).toContain("📈 *Pair:* `BTC/USD`"); // from baseOpportunity
      // The formatter uses escapeMarkdownV2(type)
      expect(message).toContain(`ℹ️ *Type:* ${escapeMarkdownV2("generic")}`);
      expect(message).toContain("💰 *Gross Metric:* `0\\.2500%`"); 
      expect(message).toContain("➡️ *Exchange 1:* `GENEX1`");
      expect(message).toContain("⬅️ *Exchange 2:* `GENEX2`");
      expect(message).toContain(`🕒 *Timestamp:* ${expectedDate}`);
    });

    it("should omit Exchange 1 if longExchange is falsy for generic type (formatter handles)", () => {
      const genericOpp = {
        ...baseOpportunity,
        type: "generic",
        longExchange: undefined, // Test formatter's robustness
      } as any;
      const message = formatOpportunityMessage(genericOpp);
      expect(message).not.toContain("➡️ *Exchange 1:*");
    });
    
    it("should omit Exchange 2 if shortExchange is falsy for generic type (formatter handles)", () => {
      const genericOpp = {
        ...baseOpportunity,
        type: "generic",
        shortExchange: undefined, // Test formatter's robustness
      } as any;
      const message = formatOpportunityMessage(genericOpp);
      expect(message).not.toContain("⬅️ *Exchange 2:*");
    });

    // Test confirms omission for potentialProfitValue === undefined (already covered by a prior test)
    it('should omit potential profit section if potentialProfitValue is undefined (verify again)', () => {
      const opportunity: ArbitrageOpportunity = { ...baseOpportunity, potentialProfitValue: undefined };
      const message = formatOpportunityMessage(opportunity);
      expect(message).not.toContain('💸 *Potential Profit:*');
    });
});

// Ensure OpportunityType is imported if used in type casts like `as OpportunityType`
// import type { OpportunityType } from '../../src/types'; // Add if tests use it.
// It was commented out from the top import. Let's check if ArbitrageOpportunity implies it.
// ArbitrageOpportunity in types.ts has type: OpportunityType. So it's implicitly used.
// For explicit casts like `type: 'fundingRate' as OpportunityType` it's not strictly needed for TS to compile if the string literal is a valid member.
// However, for clarity or stricter typing in tests, it could be imported. For now, assuming it's fine. 