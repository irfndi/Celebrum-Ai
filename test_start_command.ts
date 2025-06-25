#!/usr/bin/env tsx

/**
 * Test script to identify issues with /start command implementation
 * Focuses on DB, Profile, KV/cache, and Session management
 */

import { UserService, SessionService } from '@celebrum-ai/shared';
import type { User, NewUser } from '@celebrum-ai/db/schema/users';

// Mock environment for testing
class MockKVNamespace {
  private store = new Map<string, { value: string; expiration?: number }>();

  async put(key: string, value: string, options?: { expirationTtl?: number }): Promise<void> {
    const expiration = options?.expirationTtl ? Date.now() + (options.expirationTtl * 1000) : undefined;
    this.store.set(key, { value, expiration });
    console.log(`[KV] PUT ${key} = ${value.substring(0, 100)}...`);
  }

  async get(key: string): Promise<string | null> {
    const item = this.store.get(key);
    if (!item) {
      console.log(`[KV] GET ${key} = null (not found)`);
      return null;
    }
    
    if (item.expiration && Date.now() > item.expiration) {
      this.store.delete(key);
      console.log(`[KV] GET ${key} = null (expired)`);
      return null;
    }
    
    console.log(`[KV] GET ${key} = ${item.value.substring(0, 100)}...`);
    return item.value;
  }

  async delete(key: string): Promise<void> {
    this.store.delete(key);
    console.log(`[KV] DELETE ${key}`);
  }

  // Helper for testing
  listKeys(): string[] {
    return Array.from(this.store.keys());
  }

  clear(): void {
    this.store.clear();
  }
}

// Mock D1 Database
class MockD1Database {
  private users = new Map<string, any>();
  private nextId = 1;

  async prepare(query: string) {
    console.log(`[DB] PREPARE: ${query}`);
    return {
      bind: (...params: any[]) => {
        console.log(`[DB] BIND:`, params);
        return {
          first: async () => {
            console.log(`[DB] FIRST`);
            // Simulate finding user by telegram ID
            if (query.includes('telegram_id')) {
              const telegramId = params[0];
              const user = Array.from(this.users.values()).find(u => u.telegramId === telegramId);
              console.log(`[DB] Found user:`, user ? `ID ${user.id}` : 'null');
              return user || null;
            }
            return null;
          },
          run: async () => {
            console.log(`[DB] RUN`);
            // Simulate user creation
            if (query.includes('INSERT')) {
              const userData = {
                id: this.nextId++,
                telegramId: params.find(p => typeof p === 'string' && p.length > 5),
                firstName: params.find(p => typeof p === 'string' && p.length < 50),
                role: 'free',
                status: 'active',
                createdAt: new Date(),
                updatedAt: new Date(),
                lastActiveAt: new Date(),
                settings: {},
                apiLimits: {},
                tradingPreferences: {}
              };
              this.users.set(userData.id.toString(), userData);
              console.log(`[DB] Created user:`, userData.id);
              return { success: true, meta: { last_row_id: userData.id } };
            }
            // Simulate user update
            if (query.includes('UPDATE')) {
              const userId = params[0];
              const user = this.users.get(userId.toString());
              if (user) {
                user.updatedAt = new Date();
                user.lastActiveAt = new Date();
                console.log(`[DB] Updated user:`, userId);
                return { success: true };
              }
            }
            return { success: false };
          }
        };
      }
    };
  }

  // Helper for testing
  listUsers(): any[] {
    return Array.from(this.users.values());
  }

  clear(): void {
    this.users.clear();
    this.nextId = 1;
  }
}

// Test scenarios
async function testStartCommandFlow() {
  console.log('\n=== Testing /start Command Flow ===\n');

  const mockKV = new MockKVNamespace();
  const mockDB = new MockD1Database();

  // Initialize services
  const userService = new UserService(mockDB as any);
  const sessionService = new SessionService(mockKV as any);

  // Test data
  const telegramUser = {
    id: 123456789,
    first_name: 'John',
    last_name: 'Doe',
    username: 'johndoe',
    language_code: 'en'
  };

  console.log('📝 Test 1: New user /start command');
  console.log('=====================================');

  try {
    // Step 1: Check if user exists (should be null for new user)
    console.log('\n🔍 Step 1: Check if user exists');
    let user = await userService.findUserByTelegramId(telegramUser.id.toString());
    console.log('User found:', user ? `ID ${user.id}` : 'null');

    if (!user) {
      // Step 2: Create new user
      console.log('\n👤 Step 2: Create new user');
      const newUserData: Partial<NewUser> = {
        telegramId: telegramUser.id.toString(),
        firstName: telegramUser.first_name,
        lastName: telegramUser.last_name,
        username: telegramUser.username,
        languageCode: telegramUser.language_code,
        role: 'free',
        status: 'active',
        lastActiveAt: new Date(),
        settings: {
          notifications: true,
          theme: 'light',
          language: telegramUser.language_code || 'en'
        },
        apiLimits: {
          exchangeApis: 2,
          aiApis: 10,
          maxDailyRequests: 100
        },
        tradingPreferences: {
          percentagePerTrade: 5,
          maxConcurrentTrades: 3,
          riskTolerance: 'medium',
          autoTrade: false
        }
      };

      user = await userService.createUser(newUserData);
      console.log('User created:', user ? `ID ${user.id}` : 'failed');
    }

    if (user) {
      // Step 3: Create session
      console.log('\n🔐 Step 3: Create session');
      const session = await sessionService.createSession(user);
      console.log('Session created:', session ? session.id : 'failed');

      // Step 4: Verify session can be retrieved
      console.log('\n🔍 Step 4: Verify session retrieval');
      const retrievedSession = await sessionService.getSession(session.id);
      console.log('Session retrieved:', retrievedSession ? retrievedSession.id : 'failed');

      // Step 5: Verify session by telegram ID
      console.log('\n🔍 Step 5: Verify session by telegram ID');
      const sessionByTelegram = await sessionService.getSessionByTelegramId(user.telegramId);
      console.log('Session by telegram ID:', sessionByTelegram ? sessionByTelegram.id : 'failed');
    }

  } catch (error) {
    console.error('❌ Error in new user flow:', error);
  }

  console.log('\n📝 Test 2: Existing user /start command');
  console.log('==========================================');

  try {
    // Step 1: Find existing user
    console.log('\n🔍 Step 1: Find existing user');
    let user = await userService.findUserByTelegramId(telegramUser.id.toString());
    console.log('User found:', user ? `ID ${user.id}` : 'null');

    if (user) {
      // Step 2: Update user info
      console.log('\n📝 Step 2: Update user info');
      const updates: Partial<NewUser> = {
        firstName: 'John Updated',
        lastName: 'Doe Updated',
        lastActiveAt: new Date()
      };
      
      const updatedUser = await userService.updateUser(user.id, updates);
      console.log('User updated:', updatedUser ? `ID ${updatedUser.id}` : 'failed');

      // Step 3: Create new session (should replace old one)
      console.log('\n🔐 Step 3: Create new session');
      const newSession = await sessionService.createSession(user);
      console.log('New session created:', newSession ? newSession.id : 'failed');
    }

  } catch (error) {
    console.error('❌ Error in existing user flow:', error);
  }

  console.log('\n📊 Final State');
  console.log('================');
  console.log('KV Keys:', mockKV.listKeys());
  console.log('DB Users:', mockDB.listUsers().map(u => ({ id: u.id, telegramId: u.telegramId, firstName: u.firstName })));
}

// Test session management edge cases
async function testSessionEdgeCases() {
  console.log('\n=== Testing Session Edge Cases ===\n');

  const mockKV = new MockKVNamespace();
  const sessionService = new SessionService(mockKV as any, 5); // 5 second TTL for testing

  const mockUser = {
    id: 1,
    telegramId: '123456789',
    firstName: 'Test',
    role: 'free',
    status: 'active'
  } as User;

  console.log('📝 Test 1: Session expiration');
  console.log('==============================');

  try {
    // Create session with short TTL
    const session = await sessionService.createSession(mockUser);
    console.log('Session created:', session.id);

    // Wait for expiration
    console.log('Waiting 6 seconds for session to expire...');
    await new Promise(resolve => setTimeout(resolve, 6000));

    // Try to retrieve expired session
    const expiredSession = await sessionService.getSession(session.id);
    console.log('Expired session retrieved:', expiredSession ? 'unexpectedly found' : 'correctly null');

  } catch (error) {
    console.error('❌ Error in session expiration test:', error);
  }

  console.log('\n📝 Test 2: Session refresh');
  console.log('============================');

  try {
    // Create session
    const session = await sessionService.createSession(mockUser);
    console.log('Session created:', session.id);

    // Refresh session
    const refreshed = await sessionService.refreshSession(mockUser.telegramId);
    console.log('Session refreshed:', refreshed ? 'success' : 'failed');

    // Verify session is still valid
    const validSession = await sessionService.getSession(session.id);
    console.log('Session still valid:', validSession ? 'yes' : 'no');

  } catch (error) {
    console.error('❌ Error in session refresh test:', error);
  }
}

// Test database edge cases
async function testDatabaseEdgeCases() {
  console.log('\n=== Testing Database Edge Cases ===\n');

  const mockDB = new MockD1Database();
  const userService = new UserService(mockDB as any);

  console.log('📝 Test 1: Duplicate telegram ID');
  console.log('==================================');

  try {
    const userData: Partial<NewUser> = {
      telegramId: '123456789',
      firstName: 'First User',
      role: 'free',
      status: 'active'
    };

    // Create first user
    const user1 = await userService.createUser(userData);
    console.log('First user created:', user1 ? `ID ${user1.id}` : 'failed');

    // Try to create duplicate
    const user2 = await userService.createUser(userData);
    console.log('Duplicate user created:', user2 ? `ID ${user2.id}` : 'correctly failed');

  } catch (error) {
    console.log('Expected error for duplicate:', error.message);
  }

  console.log('\n📝 Test 2: Invalid user data');
  console.log('==============================');

  try {
    const invalidData: Partial<NewUser> = {
      // Missing required telegramId
      firstName: 'Invalid User',
      role: 'free',
      status: 'active'
    };

    const invalidUser = await userService.createUser(invalidData);
    console.log('Invalid user created:', invalidUser ? 'unexpectedly succeeded' : 'correctly failed');

  } catch (error) {
    console.log('Expected error for invalid data:', error.message);
  }
}

// Main test runner
async function runTests() {
  console.log('🚀 Starting /start Command Investigation\n');
  console.log('========================================\n');

  try {
    await testStartCommandFlow();
    await testSessionEdgeCases();
    await testDatabaseEdgeCases();

    console.log('\n✅ All tests completed');
    console.log('========================');
    console.log('\n📋 Summary of potential issues to investigate:');
    console.log('1. Session expiration handling');
    console.log('2. Database constraint violations');
    console.log('3. KV storage reliability');
    console.log('4. User profile data validation');
    console.log('5. Concurrent session creation');
    console.log('6. Error handling and recovery');

  } catch (error) {
    console.error('❌ Test runner failed:', error);
  }
}

// Run if called directly
if (require.main === module) {
  runTests().catch(console.error);
}

export { runTests, testStartCommandFlow, testSessionEdgeCases, testDatabaseEdgeCases };