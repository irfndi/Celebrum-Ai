// src/utils/formatter.ts

import type {
  ArbitrageOpportunity as TypedArbitrageOpportunity,
  ExchangeId,
} from "../types";

// Helper to escape MarkdownV2 characters
// See: https://core.telegram.org/bots/api#markdownv2-style
export function escapeMarkdownV2(text: string | number | undefined): string {
  if (text === undefined) return "N/A";
  const textStr = String(text);
  // Characters to escape: _ * [ ] ( ) ~ ` > # + - = | { } . !
  const charsToEscape = /[_*\[\]()~`>#+\-=|{}.!]/g;
  return textStr.replace(charsToEscape, "\\$&");
}

/**
 * Formats an ArbitrageOpportunity object into a MarkdownV2 string for Telegram.
 * @param opportunity The arbitrage opportunity (TypedArbitrageOpportunity from src/types.ts).
 * @returns A formatted string ready for Telegram.
 */
export function formatOpportunityMessage(
  opportunity: TypedArbitrageOpportunity
): string {
  // Destructure from TypedArbitrageOpportunity
  const {
    pair,
    longExchange,
    shortExchange,
    longRate,
    shortRate,
    rateDifference,
    netRateDifference,
    potentialProfitValue,
    timestamp,
    type,
    details,
  } = opportunity;

  // Format rates and difference as percentages with fixed precision
  const longRatePercent =
    typeof longRate === "number" ? (longRate * 100).toFixed(4) : "N/A";
  const shortRatePercent =
    typeof shortRate === "number" ? (shortRate * 100).toFixed(4) : "N/A";
  const diffPercent = (rateDifference * 100).toFixed(4);
  const netDiffPercent =
    typeof netRateDifference === "number"
      ? (netRateDifference * 100).toFixed(4)
      : undefined;

  const dateStr = new Date(timestamp).toLocaleString(); // Adjust locale/format as needed

  // Escape dynamic values
  const pairEscaped = escapeMarkdownV2(pair);
  const longExEscaped = escapeMarkdownV2(longExchange?.toUpperCase());
  const shortExEscaped = escapeMarkdownV2(shortExchange?.toUpperCase());
  const longRateEscaped = escapeMarkdownV2(longRatePercent);
  const shortRateEscaped = escapeMarkdownV2(shortRatePercent);
  const diffEscaped = escapeMarkdownV2(diffPercent);
  const netDiffEscaped = netDiffPercent
    ? escapeMarkdownV2(netDiffPercent)
    : undefined;
  const potentialProfitEscaped = escapeMarkdownV2(
    potentialProfitValue?.toFixed(2)
  ); // Assuming monetary value, 2 decimal places
  const dateEscaped = escapeMarkdownV2(dateStr);
  const detailsEscaped = details ? escapeMarkdownV2(details) : undefined;

  // Build the message using MarkdownV2 syntax
  let message = `
🚨 *Arbitrage Opportunity Detected* 🚨

📈 *Pair:* \`${pairEscaped}\``;

  if (type === "fundingRate" && longExchange && shortExchange) {
    message += `
↔️ *Action:* LONG \`${longExEscaped}\` / SHORT \`${shortExEscaped}\`

*Rates \\(Funding\\):*
   \\- Long \\(${longExEscaped}\\): \`${longRateEscaped}%\`
   \\- Short \\(${shortExEscaped}\\): \`${shortRateEscaped}%\`
💰 *Gross Difference:* \`${diffEscaped}%\``;
  } else {
    // Generic message for other types or if specific fields are missing
    message += `
ℹ️ *Type:* ${escapeMarkdownV2(type)}
💰 *Gross Metric:* \`${diffEscaped}%\``;
    if (longExchange) message += `\n➡️ *Exchange 1:* \`${longExEscaped}\``;
    if (shortExchange) message += `\n⬅️ *Exchange 2:* \`${shortExEscaped}\``;
  }

  if (netDiffEscaped) {
    message += `
💹 *Net Difference:* \`${netDiffEscaped}%\``;
  }

  if (potentialProfitEscaped && potentialProfitEscaped !== "N/A") {
    message += `
💸 *Potential Profit:* \\~$${potentialProfitEscaped}`;
  }

  if (detailsEscaped) {
    message += `
📝 *Details:* ${detailsEscaped}`;
  }

  message += `
🕒 *Timestamp:* ${dateEscaped}`;

  return message;
}