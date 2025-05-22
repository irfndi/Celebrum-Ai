import { vi } from 'vitest';

// Mock logform globally to prevent it from loading @colors/colors,
// which causes issues with 'node:os' resolution in the test environment.
vi.mock('logform');

// Mock winston globally as it has internal resolution issues for logform.
vi.mock('winston');

// Use manual CCXT mock for all tests
vi.mock('ccxt', async () => {
  const mock = await import('./__mocks__/ccxt');
  return {
    default: mock.default,
    ...mock,
  };
}); 