import type { Mock, vi } from "vitest";
import type { LoggerInterface } from "../../src/types"; // Adjust path as needed

// Copied from tests/index.test.ts (original lines approx 69-96)
export interface MockLogger extends LoggerInterface {
  /* ... all methods as vi.fn() ... */
  // Ensure all methods from LoggerInterface are here and are Mocks
  debug: Mock<(message: string, ...meta: unknown[]) => void>;
  info: Mock<(message: string, ...meta: unknown[]) => void>;
  warn: Mock<(message: string, ...meta: unknown[]) => void>;
  error: Mock<(message: string, ...meta: unknown[]) => void>;
  log: Mock<(level: string, message: string, ...meta: unknown[]) => void>;
  http?: Mock<(message: string, ...meta: unknown[]) => void>;
  verbose?: Mock<(message: string, ...meta: unknown[]) => void>;
  silly?: Mock<(message: string, ...meta: unknown[]) => void>;
  child?: Mock<(options: Record<string, unknown>) => MockLogger>;
  setLogLevel: Mock<(level: string) => void>;
  getLogLevel: Mock<() => string>;
  addError?: Mock<(error: Error, ...meta: unknown[]) => void>;
  addContext?: Mock<(key: string, value: unknown) => void>;
}

export const createMockLogger = (): MockLogger => ({
  debug: vi.fn(),
  info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
  log: vi.fn(),
  http: vi.fn(),
  verbose: vi.fn(),
  silly: vi.fn(),
  child: vi.fn().mockImplementation(() => createMockLogger()), // Important: recursive call to itself
  setLogLevel: vi.fn(),
  getLogLevel: vi.fn().mockReturnValue("info"),
  addError: vi.fn(),
  addContext: vi.fn(),
}); 