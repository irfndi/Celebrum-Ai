/// <reference types="vitest/globals" />
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PositionsManager, type ManagedPosition, type PositionData, type UpdatePositionData } from '../../src/services/positionsManager';
import type { Env } from '../../src/types';

import type {
  DurableObjectId,
  DurableObjectState,
  DurableObjectStorage,
  DurableObjectListOptions,
  DurableObjectGetOptions,
  DurableObjectTransaction,
  DurableObjectPutOptions,
} from '@cloudflare/workers-types';

const mockEnv = { POSITIONS_DO: {} } as unknown as Env;

// PositionsManager stores all positions in a Record<string, ManagedPosition> under a single key "positions".
// So, T for storage operations related to this will be Record<string, ManagedPosition>.
const POSITIONS_KEY = "positions";

const mockStorage = (): DurableObjectStorage => {
  const store = new Map<string, unknown>(); // Use unknown for broader compatibility, cast in methods

  const transactionMock: DurableObjectStorage['transaction'] = vi.fn(
    async <T>(closure: (txn: DurableObjectTransaction) => Promise<T>): Promise<T> => {
      const txnStore = new Map(store);
      const transactionObject: DurableObjectTransaction = {
        get: vi.fn().mockImplementation(async <V = unknown>(keyOrKeys: string | string[], options?: DurableObjectGetOptions): Promise<V | undefined | Map<string, V>> => {
          if (typeof keyOrKeys === 'string') return txnStore.get(keyOrKeys) as V | undefined;
          const result = new Map<string, V>();
          for (const k of keyOrKeys) { const val = txnStore.get(k); if (val !== undefined) result.set(k, val as V); }
          return result;
        }),
        put: vi.fn().mockImplementation(async <V>(keyOrEntries: string | Record<string, V>, value?: V, options?: DurableObjectPutOptions): Promise<void> => {
          if (typeof keyOrEntries === 'string') txnStore.set(keyOrEntries, value);
          else for (const [k, v] of Object.entries(keyOrEntries)) txnStore.set(k, v);
        }),
        delete: vi.fn().mockImplementation(async (keyOrKeys: string | string[], options?: DurableObjectPutOptions): Promise<boolean | number> => {
          if (Array.isArray(keyOrKeys)) { let count = 0; for (const k of keyOrKeys) if (txnStore.delete(k)) count++; return count; }
          return txnStore.delete(keyOrKeys);
        }),
        list: vi.fn().mockImplementation(async <V = unknown>(options?: DurableObjectListOptions): Promise<Map<string, V>> => {
            const listResult = new Map<string, V>();
            for (const [key, value] of txnStore.entries()) listResult.set(key, value as V);
            return listResult;
        }),
        rollback: vi.fn(() => { /* Rollback logic */ }),
        getAlarm: vi.fn().mockResolvedValue(null as number | null),
        setAlarm: vi.fn().mockResolvedValue(undefined),
        deleteAlarm: vi.fn().mockResolvedValue(undefined),
      };
      const result = await closure(transactionObject);
      for (const [key, value] of txnStore.entries()) store.set(key, value);
      return result;
    }
  );

  const transactionSyncMock: DurableObjectStorage['transactionSync'] = vi.fn(
    <T>(closure: () => T): T => {
      return closure();
    }
  );

  const onNextSessionRestoreBookmarkMock: DurableObjectStorage['onNextSessionRestoreBookmark'] = vi.fn(
    async (bookmark: string): Promise<string> => {
      return `restored-${bookmark}-mocked`;
    }
  );

  const mockedStorage: DurableObjectStorage = {
    get: vi.fn().mockImplementation(async <T = unknown>(keyOrKeys: string | string[], options?: DurableObjectGetOptions): Promise<T | undefined | Map<string, T>> => {
      if (typeof keyOrKeys === 'string') return store.get(keyOrKeys) as T | undefined;
      const result = new Map<string, T>();
      for (const k of keyOrKeys) { const val = store.get(k); if (val !== undefined) result.set(k, val as T);}
      return result;
    }),
    list: vi.fn().mockImplementation(async <T = unknown>(options?: DurableObjectListOptions): Promise<Map<string, T>> => {
      const result = new Map<string, T>();
      for (const [key, value] of store.entries()) result.set(key, value as T);
      if (store.has(POSITIONS_KEY)) {
        const allPositions = store.get(POSITIONS_KEY) as T;
        const listResult = new Map<string, T>();
        listResult.set(POSITIONS_KEY, allPositions);
        return listResult;
      }
      return new Map<string, T>();
    }),
    put: vi.fn().mockImplementation(async <T>(keyOrEntries: string | Record<string, T>, value?: T, options?: DurableObjectPutOptions): Promise<void> => {
      if (typeof keyOrEntries === 'string') store.set(keyOrEntries, value);
      else for (const [k, v] of Object.entries(keyOrEntries)) store.set(k, v);
    }),
    delete: vi.fn().mockImplementation(async (keyOrKeys: string | string[], options?: DurableObjectPutOptions): Promise<boolean | number> => {
      if (Array.isArray(keyOrKeys)) { let count = 0; for (const k of keyOrKeys) if (store.delete(k)) count++; return count; }
      return store.delete(keyOrKeys);
    }),
    deleteAll: vi.fn(async (options?: DurableObjectPutOptions): Promise<void> => { store.clear(); }),
    // biome-ignore lint/suspicious/noExplicitAny: Complex generic type for Vitest mock
    transaction: transactionMock as any, 
    sync: vi.fn().mockResolvedValue(undefined),
    getAlarm: vi.fn().mockResolvedValue(null as number | null),
    setAlarm: vi.fn().mockResolvedValue(undefined),
    deleteAlarm: vi.fn().mockResolvedValue(undefined),

    // biome-ignore lint/suspicious/noExplicitAny: Unclear type for sql from @cloudflare/workers-types
    sql: vi.fn() as any, 

    // biome-ignore lint/suspicious/noExplicitAny: Complex generic type for Vitest mock
    transactionSync: transactionSyncMock as any,
    getCurrentBookmark: vi.fn().mockResolvedValue('mock-bookmark-string'),
    getBookmarkForTime: vi.fn().mockResolvedValue('mock-bookmark-for-time-string'),
    // biome-ignore lint/suspicious/noExplicitAny: Complex generic type for Vitest mock
    onNextSessionRestoreBookmark: onNextSessionRestoreBookmarkMock as any,
  };

  return mockedStorage;
};

const mockState = (): DurableObjectState => {
  return {
    id: { toString: () => 'test-do-id', name: 'test-do', equals: vi.fn().mockReturnValue(true) } as DurableObjectId,
    storage: mockStorage(),
    waitUntil: vi.fn(),
    blockConcurrencyWhile: vi.fn(async (callback) => await callback()),
    acceptWebSocket: vi.fn(),
    getWebSockets: vi.fn().mockReturnValue([]),
    setWebSocketAutoResponse: vi.fn(),
    getWebSocketAutoResponse: vi.fn(),
    autoResponse: undefined,
    webSocketPair: undefined,
    webSocketMessage: undefined,
    webSocketError: undefined,
    getWebSocketAutoResponseTimestamp: vi.fn().mockReturnValue(null),
    setHibernatableWebSocketEventTimeout: vi.fn(),
    getHibernatableWebSocketEventTimeout: vi.fn().mockReturnValue(null),
    getTags: vi.fn().mockReturnValue([]),
    abort: vi.fn((reason?: unknown) => { throw new Error(`Durable Object aborted: ${reason}`); }),
  } as DurableObjectState;
};

describe('PositionsManager Durable Object', () => {
  let positionsManager: PositionsManager;
  let state: DurableObjectState;

beforeEach(async () => {
    state = mockState(); // Initialize state here
    await state.storage.put(POSITIONS_KEY, {});
    positionsManager = new PositionsManager(state, mockEnv);
  });

  describe('POST /positions validation', () => {
    const baseValidData: PositionData = {
      symbol: 'BTC/USDT',
      side: 'long',
      entryPrice: 50000,
      size: 1,
      exchange: 'binance_test',
    };

    const testCases = [
      { name: 'missing symbol', payload: { ...baseValidData, symbol: undefined }, field: 'symbol' },
      { name: 'empty symbol', payload: { ...baseValidData, symbol: "" }, field: 'symbol' },
      { name: 'missing side', payload: { ...baseValidData, side: undefined }, field: 'side' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'invalid side', payload: { ...baseValidData, side: 'invalid_side' as any }, field: 'side' },
      { name: 'missing entryPrice', payload: { ...baseValidData, entryPrice: undefined }, field: 'entryPrice' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'non-numeric entryPrice', payload: { ...baseValidData, entryPrice: 'not-a-number' as any }, field: 'entryPrice' },
      { name: 'zero entryPrice', payload: { ...baseValidData, entryPrice: 0 }, field: 'entryPrice' },
      { name: 'negative entryPrice', payload: { ...baseValidData, entryPrice: -100 }, field: 'entryPrice' },
      { name: 'missing size', payload: { ...baseValidData, size: undefined }, field: 'size' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'non-numeric size', payload: { ...baseValidData, size: 'not-a-number' as any }, field: 'size' },
      { name: 'negative size', payload: { ...baseValidData, size: -1 }, field: 'size' },
      { name: 'missing exchange', payload: { ...baseValidData, exchange: undefined }, field: 'exchange' },
      { name: 'empty exchange', payload: { ...baseValidData, exchange: "" }, field: 'exchange' },
    ];

    for (const tc of testCases) {
      it(`should return 400 for ${tc.name}`, async () => {
        const request = new Request('http://localhost/positions', {
          method: 'POST',
          body: JSON.stringify(tc.payload),
          headers: { 'Content-Type': 'application/json' },
        });

        const response = await positionsManager.fetch(request);
        expect(response.status).toBe(400);
        const body = await response.json();
        expect(body.message).toBe('Validation failed');
        expect(body.errors).toBeDefined();
        if (tc.field) {
          expect(body.errors[tc.field]).toBeDefined();
          expect(Array.isArray(body.errors[tc.field])).toBe(true);
          expect(body.errors[tc.field].length).toBeGreaterThan(0);
        }
      });
    }
  });

  it('should open a new position successfully', async () => {
    const newPositionData: PositionData = {
      symbol: 'BTC/USDT',
      side: 'long',
      size: 1,
      entryPrice: 50000,
      exchange: 'binance_test',
    };
    const request = new Request('http://localhost/positions', {
      method: 'POST',
      body: JSON.stringify(newPositionData),
      headers: { 'Content-Type': 'application/json' },
    });

    const response = await positionsManager.fetch(request);
    expect(response.status).toBe(201);
    const createdPosition = await response.json() as ManagedPosition;
    expect(createdPosition.id).toBeDefined();
    expect(createdPosition.symbol).toBe(newPositionData.symbol);
    expect(createdPosition.status).toBe('open');

    const storedMap = await state.storage.get<Record<string, ManagedPosition>>(POSITIONS_KEY);
    expect(storedMap).toBeDefined();
    if (storedMap) {
        expect(storedMap[createdPosition.id]).toEqual(createdPosition);
    }
  });

  it('should update an existing position', async () => {
    const newPositionData: PositionData = {
      symbol: 'ADA/USDT',
      side: 'short',
      size: 1000,
      entryPrice: 1.5,
      exchange: 'binance_us',
    };
    await state.blockConcurrencyWhile(async () => {});

    const postResponse = await positionsManager.fetch(new Request('http://localhost/positions', { method: 'POST', body: JSON.stringify(newPositionData), headers: { 'Content-Type': 'application/json' } }));
    const createdPosition = await postResponse.json() as ManagedPosition;

    const updatePayload: UpdatePositionData = {
      status: 'closed', 
      pnl: 150,
    };
    const updateRequest = new Request(`http://localhost/positions/${createdPosition.id}`, {
      method: 'PUT',
      body: JSON.stringify(updatePayload),
      headers: { 'Content-Type': 'application/json' },
    });
    const updateResponse = await positionsManager.fetch(updateRequest);
    expect(updateResponse.status).toBe(200);
    const updatedPosition = await updateResponse.json() as ManagedPosition;
    expect(updatedPosition.status).toBe('closed');
    expect(updatedPosition.pnl).toBe(150);

    const storedMap = await state.storage.get<Record<string, ManagedPosition>>(POSITIONS_KEY);
    expect(storedMap).toBeDefined();
    if (storedMap) {
        expect(storedMap[createdPosition.id]?.status).toBe('closed');
        expect(storedMap[createdPosition.id]?.pnl).toBe(150);
    }
  });

  describe('PUT /positions/:id validation', () => {
    let existingPositionId: string;
    const baseValidPostData: PositionData = {
      symbol: 'LINK/USDT',
      side: 'long',
      entryPrice: 20,
      size: 100,
      exchange: 'kraken_test',
    };

    beforeEach(async () => {
      // Create a position to update
      const request = new Request('http://localhost/positions', {
        method: 'POST',
        body: JSON.stringify(baseValidPostData),
        headers: { 'Content-Type': 'application/json' },
      });
      const response = await positionsManager.fetch(request);
      const pos = await response.json() as ManagedPosition;
      existingPositionId = pos.id;
    });

    const testCases = [
      { name: 'empty payload', payload: {}, expectedStatus: 400, messageSubString: 'No valid fields' },
      { name: 'disallowed field - symbol', payload: { symbol: 'ETH/BTC' }, expectedStatus: 400, messageSubString: 'Validation failed'},
      { name: 'disallowed field - id', payload: { id: 'new-id' }, expectedStatus: 400, messageSubString: 'Validation failed' },
      { name: 'disallowed field - entryPrice', payload: { entryPrice: 123 }, expectedStatus: 400, messageSubString: 'Validation failed' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'invalid status enum', payload: { status: 'pending' as any }, field: 'status', expectedStatus: 400, messageSubString: 'Validation failed' },
      { name: 'negative size', payload: { size: -5 }, field: 'size', expectedStatus: 400, messageSubString: 'Validation failed' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'non-numeric size', payload: { size: 'abc' as any }, field: 'size', expectedStatus: 400, messageSubString: 'Validation failed' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'non-numeric pnl', payload: { pnl: 'profit' as any }, field: 'pnl', expectedStatus: 400, messageSubString: 'Validation failed' },
      // biome-ignore lint/suspicious/noExplicitAny: Intentional type mismatch for testing validation
      { name: 'non-numeric margin', payload: { margin: 'full' as any}, field: 'margin', expectedStatus: 400, messageSubString: 'Validation failed' }
    ];

    for (const tc of testCases) {
      it(`should return ${tc.expectedStatus} for ${tc.name}`, async () => {
        const request = new Request(`http://localhost/positions/${existingPositionId}`, {
          method: 'PUT',
          body: JSON.stringify(tc.payload),
          headers: { 'Content-Type': 'application/json' },
        });

        const response = await positionsManager.fetch(request);
        expect(response.status).toBe(tc.expectedStatus);
        const body = await response.json();
        expect(body.message).toContain(tc.messageSubString);
        if (tc.field && body.errors) { // errors might not exist for 'No valid fields' case
          expect(body.errors[tc.field]).toBeDefined();
          expect(Array.isArray(body.errors[tc.field])).toBe(true);
          expect(body.errors[tc.field].length).toBeGreaterThan(0);
        }
      });
    }
  });

  it('should close (delete) an existing position', async () => {
    const newPositionData: PositionData = {
      symbol: 'DOT/USDT',
      side: 'long',
      size: 50,
      entryPrice: 20,
      exchange: 'ftx_rip',
    };
    await state.blockConcurrencyWhile(async () => {});

    const postResponse = await positionsManager.fetch(new Request('http://localhost/positions', { method: 'POST', body: JSON.stringify(newPositionData), headers: { 'Content-Type': 'application/json' } }));
    const createdPosition = await postResponse.json() as ManagedPosition;

    const deleteRequest = new Request(`http://localhost/positions/${createdPosition.id}`, { method: 'DELETE' });
    const deleteResponse = await positionsManager.fetch(deleteRequest);
    expect(deleteResponse.status).toBe(200);
    const responseText = await deleteResponse.text();
    expect(responseText).toBe('Position Closed');

    const storedMap = await state.storage.get<Record<string, ManagedPosition>>(POSITIONS_KEY);
    expect(storedMap).toBeDefined();
    if (storedMap) {
        expect(storedMap[createdPosition.id]).toBeUndefined();
    }
  });
});
