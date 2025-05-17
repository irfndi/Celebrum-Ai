// src/utils/formatter.ts

import type {
  ArbitrageOpportunity as TypedArbitrageOpportunity,
  ExchangeId,
} from "../types";

// Helper to escape MarkdownV2 characters
// See: https://core.telegram.org/bots/api#markdownv2-style
function escapeMarkdownV2(text: string | number | undefined): string {
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
    pairSymbol,
    longExchange,
    shortExchange,
    longRate,
    shortRate,
    grossProfitMetric, // This is the primary rateDifference for funding rates
    netProfitMetric, // This would be the netRateDifference
    potentialProfitValue, // For potential profit display
    timestamp,
    type,
    details,
  } = opportunity;

  // Format rates and difference as percentages with fixed precision
  // Ensure rates are numbers before calling toFixed, provide fallback if not (though they should be for fundingRate type)
  const longRatePercent =
    typeof longRate === "number" ? (longRate * 100).toFixed(4) : "N/A";
  const shortRatePercent =
    typeof shortRate === "number" ? (shortRate * 100).toFixed(4) : "N/A";
  const diffPercent = (grossProfitMetric * 100).toFixed(4);
  const netDiffPercent =
    typeof netProfitMetric === "number"
      ? (netProfitMetric * 100).toFixed(4)
      : undefined;

  const dateStr = new Date(timestamp).toLocaleString(); // Adjust locale/format as needed

  // Escape dynamic values
  const pairEscaped = escapeMarkdownV2(pairSymbol);
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
💸 *Potential Profit:* \`${potentialProfitEscaped}\` USDT`; // Assuming USDT or a common quote
  }
  if (detailsEscaped) {
    message += `\n📝 *Details:* ${detailsEscaped}`;
  }

  message += `

🕒 *Detected At:* ${dateEscaped}
`;

  // Optional: Command example (kept generic for now)
  if (type === "fundingRate" && longExchange && shortExchange) {
    message += `
*To execute manually \\(Example\\):*
\`/execute ${pairEscaped.replace(/[\/\\]/g, "_")} ${longExEscaped} ${shortExEscaped} 0\\.1 10\`
\\(Replace 0\\.1 with size, 10 with leverage\\)
  `;
  }

  return message.trim(); // Trim leading/trailing whitespace
}

// Add other formatting helpers as needed (e.g., for positions, balances)
