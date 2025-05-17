// Extend this interface to type your test environment variables
import type { Env as MainEnv } from '../../src/types';

declare module "cloudflare:test" {
  interface ProvidedEnv extends MainEnv {
    // Example: Add your KV Namespaces, D1 Databases, R2 Buckets, etc.
    // MY_KV_NAMESPACE: KVNamespace;
    // MY_D1_DATABASE: D1Database;
    // MY_R2_BUCKET: R2Bucket;
    TEST_KV: KVNamespace;
  }
}
