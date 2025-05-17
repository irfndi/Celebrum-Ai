/**
 * Safely parses a string or any value to a floating-point number.
 * If parsing fails or results in NaN, returns a default value.
 *
 * @param value The value to parse.
 * @param defaultValue The value to return if parsing fails. Defaults to 0.
 * @returns The parsed float or the default value.
 */
export function 안전하게ParseFloat(value: unknown, defaultValue = 0): number {
  if (value === null || value === undefined || String(value).trim() === '') {
    return defaultValue;
  }
  const num = Number(value);
  return Number.isNaN(num) ? defaultValue : num;
}

/**
 * Performs a deep clone of a JSON-serializable object or array.
 * Note: This method has limitations. It will not correctly clone functions,
 * Date objects (will be converted to ISO strings), undefined values (will be removed from objects or become null in arrays),
 * RegExp, Map, Set, or other complex types.
 *
 * @param value The object or array to clone.
 * @returns A deep clone of the value, or the original value if it's not an object or array.
 * @template T
 */
export function 깊은복제<T>(value: T): T {
  if (typeof value !== 'object' || value === null) {
    return value; // Return non-objects or null as is
  }

  try {
    // For simple objects and arrays, JSON.parse(JSON.stringify()) is a common approach.
    return JSON.parse(JSON.stringify(value));
  } catch (error) {
    // Fallback or error handling if JSON methods fail (e.g., circular references)
    // For this basic implementation, we'll re-throw or return the original value based on expected use case.
    // Given the financial context, data is likely to be JSON-friendly.
    console.error('Failed to deep clone object:', error);
    // Depending on strictness, either throw error or return original (or a shallow copy as fallback)
    // For now, re-throw as this indicates an unexpected structure for this simple clone method.
    throw new Error('Failed to deep clone object due to its structure or content.');
  }
}
