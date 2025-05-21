import type {
  DurableObjectId,
  DurableObjectJurisdiction,
  DurableObjectNamespace,
  DurableObjectStub,
  KVNamespace,
  KVNamespaceGetOptions,
  KVNamespaceGetWithMetadataResult,
  KVNamespaceListKey,
  KVNamespaceListOptions,
  KVNamespaceListResult,
  KVNamespacePutOptions,
  Socket,
} from "@cloudflare/workers-types";
import { ReadableStream } from "@cloudflare/workers-types"; // Separate import for ReadableStream if not covered by main types
import { vi } from "vitest";

// --- BEGIN KV/DO MOCK HELPERS (Moved from tests/index.test.ts) ---

// Interfaces and Types for KV Mock
export type MockKVNamespaceGetOptionsType = "text" | "json" | "arrayBuffer" | "stream";
export interface MockKVNamespaceGetOptions {
  type?: MockKVNamespaceGetOptionsType;
  // cacheTtl?: number; // Not mocked as it's not used by processGetOptions
}

export interface MockKVNamespace extends KVNamespace {
  text: (key: string) => Promise<string | null>;
  json: <T = unknown>(key: string) => Promise<T | null>;
  arrayBuffer: (key: string) => Promise<ArrayBuffer | null>;
  stream: (key: string) => Promise<ReadableStream | null>;
}

export interface KVStoreEntry {
  value: string | ArrayBuffer | ReadableStream;
  expiration?: number;
  metadata?: unknown;
}

export interface KVGetWithMetadataResult<T, M> {
  value: T | null;
  metadata: M | null;
  cacheStatus: "HIT" | "MISS" | "STALE" | null;
}

export type KVNamespaceReadType = "text" | "json" | "arrayBuffer" | "stream";

// Helper function to process get options
export function processGetOptions(
  optionsOrType?:
    | KVNamespaceReadType
    | Partial<KVNamespaceGetOptions<KVNamespaceReadType>>
): KVNamespaceReadType {
  if (typeof optionsOrType === "string") {
    const validTypes: KVNamespaceReadType[] = ["text", "json", "arrayBuffer", "stream"];
    if (validTypes.includes(optionsOrType as KVNamespaceReadType)) {
      return optionsOrType as KVNamespaceReadType;
    }
    return "text";
  }
  if (optionsOrType && typeof optionsOrType === "object" && optionsOrType.type) {
    const type = optionsOrType.type;
    const validTypes: KVNamespaceReadType[] = ["text", "json", "arrayBuffer", "stream"];
    if (validTypes.includes(type)) {
      return type;
    }
  }
  return "text";
}

// Helper function to convert value based on type
export async function convertValue(value: string, type: "text"): Promise<string | null>;
export async function convertValue(value: string, type: "arrayBuffer"): Promise<ArrayBuffer | null>;
export async function convertValue(value: string, type: "stream"): Promise<ReadableStream | null>;
export async function convertValue<T = unknown>(value: string, type: "json"): Promise<T | null>;
export async function convertValue<T = unknown>(
  value: string,
  type: "text" | "json" | "arrayBuffer" | "stream"
): Promise<T | string | ArrayBuffer | ReadableStream | null> {
  switch (type) {
    case "text":
      return value;
    case "json":
      try {
        return JSON.parse(value) as T;
      } catch (e) {
        console.error("Mock KV: Failed to parse JSON", e);
        return null;
      }
    case "arrayBuffer":
      return new TextEncoder().encode(value).buffer as ArrayBuffer;
    case "stream":
      return new ReadableStream({
        start(controller) {
          controller.enqueue(new TextEncoder().encode(value));
          controller.close();
        },
      });
    default:
      console.error(`Mock KV: Unexpected type in convertValue: ${type}.`);
      return null;
  }
}

export async function streamToArrayBuffer(stream: ReadableStream<Uint8Array>): Promise<ArrayBuffer> {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let totalLength = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    if (value) {
      chunks.push(value);
      totalLength += value.length;
    }
  }
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result.buffer;
}

export const createSimpleMockKvNamespace = (): KVNamespace => {
  const store = new Map<string, KVStoreEntry>();
  const metadataStore = new Map<string, unknown>();
  const expirationStore = new Map<string, number>();

  const kvNamespaceImplInternal: KVNamespace = {
    async get<ExpectedValue = unknown>(
      key: string | string[],
      optionsOrType?: KVNamespaceReadType | Partial<KVNamespaceGetOptions<KVNamespaceReadType>>
    ): Promise<string | ExpectedValue | ArrayBuffer | ReadableStream | null | Map<string, string | ExpectedValue | ArrayBuffer | ReadableStream | null>> {
      if (Array.isArray(key)) {
        throw new Error("Mock KV: Batch get operation not implemented.");
      }
      const valueEntry = store.get(key);
      if (valueEntry === undefined) return null;

      const effectiveExpiration = expirationStore.get(key) ?? valueEntry.expiration;
      if (effectiveExpiration && Date.now() >= effectiveExpiration * 1000) {
        store.delete(key);
        metadataStore.delete(key);
        expirationStore.delete(key);
        return null;
      }
      const type = processGetOptions(optionsOrType);
      if (type === "json") return convertValue<ExpectedValue>(valueEntry.value as string, "json");
      if (type === "text") return convertValue(valueEntry.value as string, "text");
      if (type === "arrayBuffer") return convertValue(valueEntry.value as string, "arrayBuffer");
      return convertValue(valueEntry.value as string, "stream");
    },
    getWithMetadata: vi.fn().mockImplementation(
      async <Value = unknown, Metadata = unknown>(
        key: string,
        optionsOrType?: "text" | "json" | "arrayBuffer" | "stream" | Partial<KVNamespaceGetOptions<undefined>>
      ): Promise<KVNamespaceGetWithMetadataResult<Value, Metadata>> => {
        const entry = store.get(key);
        let type: "text" | "json" | "arrayBuffer" | "stream" = "json";
        if (typeof optionsOrType === "string") type = optionsOrType;
        else if (optionsOrType && typeof optionsOrType === "object" && optionsOrType.type) {
          const validTypes: Array<"text" | "json" | "arrayBuffer" | "stream"> = ["text", "json", "arrayBuffer", "stream"];
          const optionType = optionsOrType.type as string;
          if (validTypes.includes(optionType as "text" | "json" | "arrayBuffer" | "stream")) type = optionType as typeof type;
        }
        if (!entry) return { value: null, metadata: null, cacheStatus: "MISS" };
        let returnValue: Value;
        switch (type) {
          case "text": returnValue = entry.value as string as unknown as Value; break;
          case "json":
            try { returnValue = JSON.parse(entry.value as string) as Value; }
            catch (e) { console.error("Mock KV JSON parse error:", e); return { value: null, metadata: (entry.metadata as Metadata) || null, cacheStatus: "MISS" }; }
            break;
          case "arrayBuffer": returnValue = new TextEncoder().encode(entry.value as string).buffer as unknown as Value; break;
          case "stream": {
            const streamValue = entry.value as string;
            returnValue = new ReadableStream({ start(controller) { controller.enqueue(new TextEncoder().encode(streamValue)); controller.close(); } }) as unknown as Value;
            break;
          }
          default: returnValue = entry.value as unknown as Value;
        }
        return { value: returnValue, metadata: (entry.metadata as Metadata) || null, cacheStatus: "HIT" };
      }
    ),
    async put(
      key: string,
      valueToPut: string | ArrayBuffer | ArrayBufferView | ReadableStream,
      options?: KVNamespacePutOptions
    ): Promise<void> {
      let stringValue: string;
      if (valueToPut instanceof ArrayBuffer || ArrayBuffer.isView(valueToPut)) {
        const buffer = valueToPut instanceof ArrayBuffer ? valueToPut : (valueToPut as ArrayBufferView).buffer;
        stringValue = new TextDecoder().decode(buffer as ArrayBuffer);
      } else if (valueToPut instanceof ReadableStream) {
        const buffer = await streamToArrayBuffer(valueToPut as ReadableStream<Uint8Array>);
        stringValue = new TextDecoder().decode(buffer);
      } else {
        stringValue = valueToPut as string;
      }
      let finalExpiration: number | undefined = options?.expiration;
      if (options?.expirationTtl) finalExpiration = Math.floor(Date.now() / 1000) + options.expirationTtl;
      store.set(key, { value: stringValue, metadata: options?.metadata, expiration: finalExpiration });
      if (options?.metadata !== undefined) metadataStore.set(key, options.metadata);
      else if (options?.metadata === null) metadataStore.delete(key);
      if (finalExpiration !== undefined) expirationStore.set(key, finalExpiration);
      else expirationStore.delete(key);
    },
    async delete(keys: string | string[]): Promise<void> {
      const keysToDelete = Array.isArray(keys) ? keys : [keys];
      for (const key of keysToDelete) {
        store.delete(key);
        metadataStore.delete(key);
        expirationStore.delete(key);
      }
    },
    async list<Metadata = unknown>(
      options?: KVNamespaceListOptions
    ): Promise<KVNamespaceListResult<Metadata, string>> {
      const prefix = options?.prefix ?? "";
      const limit = options?.limit ?? 1000;
      const cursor = options?.cursor;
      const allMatchingKeys: KVNamespaceListKey<Metadata>[] = [];
      const sortedStoreKeys = Array.from(store.keys()).sort();
      let pastCursor = cursor === undefined;
      for (const key of sortedStoreKeys) {
        if (!pastCursor && key === cursor) { pastCursor = true; continue; }
        if (!pastCursor) continue;
        if (key.startsWith(prefix)) {
          const valueEntry = store.get(key);
          if (!valueEntry) continue;
          const effectiveExpiration = expirationStore.get(key) ?? valueEntry.expiration;
          if (effectiveExpiration && Date.now() >= effectiveExpiration * 1000) {
            store.delete(key); metadataStore.delete(key); expirationStore.delete(key);
            continue;
          }
          allMatchingKeys.push({
            name: key,
            expiration: effectiveExpiration,
            metadata: (metadataStore.get(key) ?? valueEntry.metadata) as Metadata | undefined,
          });
        }
      }
      const keysSlice = allMatchingKeys.slice(0, limit);
      const list_complete = keysSlice.length === allMatchingKeys.length;
      const nextCursor = list_complete ? undefined : keysSlice[keysSlice.length - 1]?.name;
      return {
        keys: keysSlice,
        list_complete: list_complete,
        cursor: nextCursor,
      } as KVNamespaceListResult<Metadata, string>;
    },
  };
  return kvNamespaceImplInternal as KVNamespace;
};

export const createMockDurableObjectNamespace = (): DurableObjectNamespace => ({
  newUniqueId: vi.fn(),
  idFromName: vi.fn(),
  idFromString: vi.fn(),
  get: vi.fn().mockImplementation(
    (id: DurableObjectId): DurableObjectStub => ({
      id,
      name: undefined,
      fetch: vi.fn().mockResolvedValue(new Response("Mock DO Response")),
      connect: vi.fn().mockImplementation((): Socket => {
        return {
          send: vi.fn(),
          close: vi.fn(),
          accept: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
          dispatchEvent: vi.fn(),
        } as unknown as Socket;
      }),
    })
  ),
  jurisdiction: vi.fn(
    (_jurisdictionParam: DurableObjectJurisdiction): DurableObjectNamespace =>
      createMockDurableObjectNamespace()
  ),
});

// --- END KV/DO MOCK HELPERS --- 