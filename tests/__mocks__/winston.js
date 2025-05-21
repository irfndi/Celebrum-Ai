import { vi } from 'vitest';

const mockLoggerInstance = {
  error: vi.fn(),
  warn: vi.fn(),
  info: vi.fn(),
  http: vi.fn(),
  verbose: vi.fn(),
  debug: vi.fn(),
  silly: vi.fn(),
  // Add other methods/properties of a winston logger instance that are used
  // For example, if level is accessed:
  level: 'info', 
  // If transports are accessed or modified:
  transports: [],
  // Mock the 'on' method if it's used for event handling
  on: vi.fn(),
  // Mock 'exitOnError' if it's used
  exitOnError: false,
};

const format = {
  combine: vi.fn((...args) => args.join(' ')), // Simplified combine
  timestamp: vi.fn(() => 'mockTimestamp'),
  printf: vi.fn((cb) => cb({ level: 'info', message: 'mockMessage', timestamp: 'mockTimestamp' })),
  align: vi.fn(() => 'mockAlign'),
  json: vi.fn(() => 'mockJson'),
  colorize: vi.fn(() => 'mockColorize'), // Mock colorize as it's conditionally imported
};

const transports = {
  Console: vi.fn().mockImplementation(() => ({
    // Mock any properties or methods of a Console transport instance if needed
    on: vi.fn(),
    level: 'info',
  })),
  // Mock other transport types if used (e.g., File)
};

const createLogger = vi.fn(() => mockLoggerInstance);

// Export all the parts that src/utils/logger.ts imports from 'winston'
export {
  createLogger,
  format,
  transports,
  // Export any other specific winston exports if your logger uses them directly
  // For example, if your logger uses winston.Logger directly for type annotations
  // or other winston utility functions.
};

// You might also need to mock the default export if 'import winston from "winston"' is used
// export default { createLogger, format, transports }; 