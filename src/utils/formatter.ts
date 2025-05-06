// src/utils/formatter.ts

import type { ArbitrageOpportunity } from '../types';

// Helper to escape MarkdownV2 characters
// See: https://core.telegram.org/bots/api#markdownv2-style
function escapeMarkdownV2(text: string | number): string {
  const textStr = String(text);
  // Characters to escape: _ * [ ] ( ) ~ ` > # + - = | { } . !
  const charsToEscape = /[_*\[\]()~`>#+\-=|{}.!\\]/g;
  return textStr.replace(charsToEscape, '\\$&');
}

/**
 * Formats an ArbitrageOpportunity object into a MarkdownV2 string for Telegram.
 * @param opportunity The arbitrage opportunity.
 * @returns A formatted string ready for Telegram.
 */
export function formatOpportunityMessage(opportunity: ArbitrageOpportunity): string {
  const { pair, longExchange, shortExchange, longRate, shortRate, rateDifference, timestamp } = opportunity;

  // Format rates and difference as percentages with fixed precision
  const longRatePercent = (longRate * 100).toFixed(4);
  const shortRatePercent = (shortRate * 100).toFixed(4);
  const diffPercent = (rateDifference * 100).toFixed(4);
  const dateStr = new Date(timestamp).toLocaleString(); // Adjust locale/format as needed

  // Escape dynamic values
  const pairEscaped = escapeMarkdownV2(pair);
  const longExEscaped = escapeMarkdownV2(longExchange.toUpperCase());
  const shortExEscaped = escapeMarkdownV2(shortExchange.toUpperCase());
  const longRateEscaped = escapeMarkdownV2(longRatePercent);
  const shortRateEscaped = escapeMarkdownV2(shortRatePercent);
  const diffEscaped = escapeMarkdownV2(diffPercent);
  const dateEscaped = escapeMarkdownV2(dateStr);

  // Build the message using MarkdownV2 syntax
  const message = `
🚨 *Arbitrage Opportunity Detected* 🚨

📈 *Pair:* \`${pairEscaped}\`
↔️ *Action:* LONG \`${longExEscaped}\` / SHORT \`${shortExEscaped}\`

*Rates \\(Funding\\):*
   \\- Long \\(${longExEscaped}\\): \`${longRateEscaped}%\`
   \\- Short \\(${shortExEscaped}\\): \`${shortRateEscaped}%\`
💰 *Difference:* \`${diffEscaped}%\`

🕒 *Detected At:* ${dateEscaped}

*To execute manually \\(Example\\):*
\`/execute ${pairEscaped.replace(/[\/\\]/g, '_')} ${longExEscaped} ${shortExEscaped} 0\\.1 10\`
\\(Replace 0\\.1 with size, 10 with leverage\\)
  `;

  return message.trim(); // Trim leading/trailing whitespace
}

// Add other formatting helpers as needed (e.g., for positions, balances)
