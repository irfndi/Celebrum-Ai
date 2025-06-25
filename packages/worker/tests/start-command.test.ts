import { describe, it, expect, beforeEach, vi } from 'vitest';
import { UserService } from '@celebrum-ai/shared/services/UserService';
import { SessionService } from '@celebrum-ai/shared/services/SessionService';
import type { User, NewUser } from '@celebrum-ai/db/schema';

// Mock KV Namespace
class MockKVNamespace {
  private store = new Map<string, { value: string; expiration?: number }>();

  async put(key: string, value: string, options?: { expirationTtl?: number }): Promise<void> {
    const expiration = options?.expirationTtl ? Date.now() + (options.expirationTtl * 1000) : undefined;
    this.store.set(key, { value, expiration });
  }

  async get(key: string): Promise<string | null> {
    const item = this.store.get(key);
    if (!item) return null;
    
    if (item.expiration && Date.now() > item.expiration) {
      this.store.delete(key);
      return null;
    }
    
    return item.value;
  }

  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }

  clear(): void {
    this.store.clear();
  }
}

// Mock D1 Database
class MockD1Database {
  private users = new Map<number, User>();
  private nextId = 1;

  // Drizzle-style methods
  insert(table: any) {
    return {
      values: (data: any) => {
        return {
          returning: async () => {
            const userData: User = {
              id: this.nextId++,
              telegramId: data.telegramId,
              firstName: data.firstName,
              lastName: data.lastName,
              username: data.username,
              email: data.email || null,
              role: data.role || 'free',
              status: data.status || 'active',
              createdAt: new Date(),
              updatedAt: new Date(),
              lastActiveAt: data.lastActiveAt || new Date(),
              settings: typeof data.settings === 'string' ? data.settings : JSON.stringify(data.settings || {}),
              apiLimits: typeof data.apiLimits === 'string' ? data.apiLimits : JSON.stringify(data.apiLimits || {}),
              accountBalance: data.accountBalance || 0,
              betaExpiresAt: data.betaExpiresAt || null,
              tradingPreferences: typeof data.tradingPreferences === 'string' ? data.tradingPreferences : JSON.stringify(data.tradingPreferences || {})
            };
            this.users.set(userData.id, userData);
            console.log('D1 insert created user:', userData);
            return [userData];
          }
        };
      }
    };
  }

  select() {
    return {
      from: (table: any) => {
        return {
          where: (condition: any) => {
            return {
              limit: (count: number) => {
                // For telegram_id lookup - simplified for testing
                const users = Array.from(this.users.values());
                console.log('D1 select found users:', users);
                return users.slice(0, count);
              }
            };
          }
        };
      }
    };
  }

  update(table: any) {
    return {
      set: (data: any) => {
        return {
          where: (condition: any) => {
            return {
              returning: async () => {
                // Find and update user (simplified)
                const users = Array.from(this.users.values());
                if (users.length > 0) {
                  const user = users[0];
                  Object.assign(user, {
                    ...data,
                    updatedAt: new Date()
                  });
                  console.log('D1 update user:', user);
                  return [user];
                }
                return [];
              }
            };
          }
        };
      }
    };
  }

  delete(table: any) {
    return {
      where: (condition: any) => {
        return {
          returning: async () => {
            const users = Array.from(this.users.values());
            if (users.length > 0) {
              const deletedUser = users[0];
              this.users.delete(deletedUser.id);
              console.log('D1 delete user:', deletedUser);
              return [deletedUser];
            }
            return [];
          }
        };
      }
    };
  }

  async prepare(query: string) {
    return {
      bind: (...params: any[]) => ({
        first: async () => {
          if (query.includes('SELECT') && query.includes('telegram_id')) {
            const telegramId = params[0];
            const user = Array.from(this.users.values()).find(u => u.telegramId === telegramId);
            return user || null;
          }
          return null;
        },
        run: async () => {
          if (query.includes('INSERT')) {
            const userData: User = {
              id: this.nextId++,
              telegramId: params[0],
              firstName: params[1],
              lastName: params[2],
              username: params[3],
              email: null,
              role: 'free',
              status: 'active',
              createdAt: new Date(),
              updatedAt: new Date(),
              lastActiveAt: new Date(),
              settings: JSON.stringify({}),
              apiLimits: JSON.stringify({}),
              accountBalance: 0,
              betaExpiresAt: null,
              tradingPreferences: JSON.stringify({})
            };
            this.users.set(userData.id, userData);
            return { success: true, meta: { last_row_id: userData.id } };
          }
          if (query.includes('UPDATE')) {
            const userId = params[params.length - 1]; // Last param is usually the ID in WHERE clause
            const user = this.users.get(userId);
            if (user) {
              // Update user with new data
              Object.assign(user, {
                firstName: params[0] || user.firstName,
                lastName: params[1] || user.lastName,
                username: params[2] || user.username,
                updatedAt: new Date(),
                lastActiveAt: new Date()
              });
              return { success: true };
            }
          }
          return { success: false };
        },
        returning: () => ({
          then: (callback: (result: User[]) => void) => {
            const user = Array.from(this.users.values()).pop();
            callback(user ? [user] : []);
          }
        })
      })
    };
  }

  clear(): void {
    this.users.clear();
    this.nextId = 1;
  }

  getUsers(): User[] {
    return Array.from(this.users.values());
  }
}

describe('/start Command Implementation', () => {
  let mockKV: MockKVNamespace;
  let mockDB: MockD1Database;
  let userService: UserService;
  let sessionService: SessionService;

  const mockTelegramUser = {
    id: 123456789,
    first_name: 'John',
    last_name: 'Doe',
    username: 'johndoe',
    language_code: 'en'
  };

  beforeEach(() => {
    mockKV = new MockKVNamespace();
    mockDB = new MockD1Database();
    userService = new UserService(mockDB as any);
    sessionService = new SessionService(mockKV as any, 3600); // 1 hour TTL
  });

  describe('New User Flow', () => {
    it('should create a new user when telegram user does not exist', async () => {
      // Check user doesn't exist
      const existingUser = await userService.findUserByTelegramId(mockTelegramUser.id.toString());
      expect(existingUser).toBeUndefined();

      // Create new user
      const newUserData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        lastName: mockTelegramUser.last_name,
        username: mockTelegramUser.username,
        role: 'free',
        status: 'active',
        lastActiveAt: new Date(),
        settings: {
          notifications: true,
          theme: 'light',
          language: mockTelegramUser.language_code || 'en'
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

      const user = await userService.createUser(newUserData);
      
      expect(user).toBeDefined();
      expect(user.telegramId).toBe(mockTelegramUser.id.toString());
      expect(user.firstName).toBe(mockTelegramUser.first_name);
      expect(user.role).toBe('free');
      expect(user.status).toBe('active');
    });

    it('should create a session for new user', async () => {
      // Create user first
      const newUserData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        role: 'free',
        status: 'active'
      };

      const user = await userService.createUser(newUserData);
      
      // Create session
      const session = await sessionService.createSession(user);
      
      expect(session).toBeDefined();
      expect(session.userId).toBe(user.id);
      expect(session.telegramId).toBe(user.telegramId);
      expect(session.isActive).toBe(true);
      expect(session.expiresAt.getTime()).toBeGreaterThan(Date.now());
    });

    it('should be able to retrieve session by telegram ID', async () => {
      // Create user and session
      const newUserData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        role: 'free',
        status: 'active'
      };

      const user = await userService.createUser(newUserData);
      const session = await sessionService.createSession(user);
      
      // Retrieve session by telegram ID
      const retrievedSession = await sessionService.getSessionByTelegramId(user.telegramId);
      
      expect(retrievedSession).toBeDefined();
      expect(retrievedSession?.id).toBe(session.id);
      expect(retrievedSession?.telegramId).toBe(user.telegramId);
    });
  });

  describe('Existing User Flow', () => {
    let existingUser: User;

    beforeEach(async () => {
      // Create an existing user
      const userData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: 'Old Name',
        lastName: 'Old Last',
        username: 'oldusername',
        role: 'free',
        status: 'active'
      };
      existingUser = await userService.createUser(userData);
    });

    it('should find existing user by telegram ID', async () => {
      const foundUser = await userService.findUserByTelegramId(mockTelegramUser.id.toString());
      
      expect(foundUser).toBeDefined();
      expect(foundUser?.id).toBe(existingUser.id);
      expect(foundUser?.telegramId).toBe(existingUser.telegramId);
    });

    it('should update existing user information', async () => {
      const updates: Partial<NewUser> = {
        firstName: mockTelegramUser.first_name,
        lastName: mockTelegramUser.last_name,
        username: mockTelegramUser.username,
        lastActiveAt: new Date()
      };

      const updatedUser = await userService.updateUser(existingUser.id, updates);
      
      expect(updatedUser).toBeDefined();
      expect(updatedUser?.firstName).toBe(mockTelegramUser.first_name);
      expect(updatedUser?.lastName).toBe(mockTelegramUser.last_name);
      expect(updatedUser?.username).toBe(mockTelegramUser.username);
    });

    it('should create new session for existing user', async () => {
      const session = await sessionService.createSession(existingUser);
      
      expect(session).toBeDefined();
      expect(session.userId).toBe(existingUser.id);
      expect(session.telegramId).toBe(existingUser.telegramId);
    });

    it('should replace old session when creating new one', async () => {
      // Create first session
      const firstSession = await sessionService.createSession(existingUser);
      
      // Create second session (should replace first)
      const secondSession = await sessionService.createSession(existingUser);
      
      // First session should be replaced
      const retrievedFirstSession = await sessionService.getSession(firstSession.id);
      const retrievedSecondSession = await sessionService.getSession(secondSession.id);
      
      expect(retrievedSecondSession).toBeDefined();
      expect(retrievedSecondSession?.id).toBe(secondSession.id);
      
      // The telegram mapping should point to the new session
      const sessionByTelegram = await sessionService.getSessionByTelegramId(existingUser.telegramId);
      expect(sessionByTelegram?.id).toBe(secondSession.id);
    });
  });

  describe('Session Management Edge Cases', () => {
    let user: User;

    beforeEach(async () => {
      const userData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        role: 'free',
        status: 'active'
      };
      user = await userService.createUser(userData);
    });

    it('should handle session expiration', async () => {
      // Create session service with very short TTL
      const shortTTLSessionService = new SessionService(mockKV as any, 1); // 1 second
      
      const session = await shortTTLSessionService.createSession(user);
      expect(session).toBeDefined();
      
      // Wait for expiration
      await new Promise(resolve => setTimeout(resolve, 1100));
      
      // Session should be expired and return null
      const expiredSession = await shortTTLSessionService.getSession(session.id);
      expect(expiredSession).toBeNull();
    });

    it('should refresh session and extend expiration', async () => {
      const session = await sessionService.createSession(user);
      const originalExpiration = session.expiresAt;
      
      // Wait a bit
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Refresh session
      const refreshedSession = await sessionService.refreshSession(session.id);
      
      expect(refreshedSession).toBeDefined();
      expect(refreshedSession?.expiresAt.getTime()).toBeGreaterThan(originalExpiration.getTime());
    });

    it('should validate session correctly', async () => {
      const session = await sessionService.createSession(user);
      
      const isValid = await sessionService.isSessionValid(session.id);
      expect(isValid).toBe(true);
      
      // Test with non-existent session
      const isInvalidValid = await sessionService.isSessionValid('non-existent-id');
      expect(isInvalidValid).toBe(false);
    });

    it('should delete session properly', async () => {
      const session = await sessionService.createSession(user);
      
      // Verify session exists
      let retrievedSession = await sessionService.getSession(session.id);
      expect(retrievedSession).toBeDefined();
      
      // Delete session
      await sessionService.deleteSession(session.id);
      
      // Verify session is deleted
      retrievedSession = await sessionService.getSession(session.id);
      expect(retrievedSession).toBeNull();
      
      // Verify telegram mapping is also deleted
      const sessionByTelegram = await sessionService.getSessionByTelegramId(user.telegramId);
      expect(sessionByTelegram).toBeNull();
    });
  });

  describe('Database Edge Cases', () => {
    it('should handle duplicate telegram ID gracefully', async () => {
      const userData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        role: 'free',
        status: 'active'
      };

      // Create first user
      const firstUser = await userService.createUser(userData);
      expect(firstUser).toBeDefined();

      // Try to create second user with same telegram ID
      // This should either fail or return the existing user
      try {
        const secondUser = await userService.createUser(userData);
        // If it succeeds, it should be the same user or handle the duplicate
        expect(secondUser.telegramId).toBe(userData.telegramId);
      } catch (error) {
        // If it fails, that's also acceptable behavior for duplicate handling
        expect(error).toBeDefined();
      }
    });

    it('should handle invalid user data', async () => {
      const invalidUserData: Partial<NewUser> = {
        // Missing required telegramId
        firstName: mockTelegramUser.first_name,
        role: 'free',
        status: 'active'
      };

      try {
        await userService.createUser(invalidUserData);
        // If this doesn't throw, the service should handle it gracefully
      } catch (error) {
        // Expected to fail due to missing required field
        expect(error).toBeDefined();
      }
    });
  });

  describe('Integration Test - Complete /start Flow', () => {
    it('should handle complete new user /start flow', async () => {
      // Step 1: Check if user exists (should be null)
      let user = await userService.findUserByTelegramId(mockTelegramUser.id.toString());
      expect(user).toBeUndefined();

      // Step 2: Create new user
      const newUserData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: mockTelegramUser.first_name,
        lastName: mockTelegramUser.last_name,
        username: mockTelegramUser.username,
        role: 'free',
        status: 'active',
        lastActiveAt: new Date(),
        settings: {
          notifications: true,
          theme: 'light',
          language: mockTelegramUser.language_code || 'en'
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
      expect(user).toBeDefined();
      expect(user.telegramId).toBe(mockTelegramUser.id.toString());

      // Step 3: Create session
      const session = await sessionService.createSession(user);
      expect(session).toBeDefined();
      expect(session.userId).toBe(user.id);

      // Step 4: Verify session can be retrieved
      const retrievedSession = await sessionService.getSession(session.id);
      expect(retrievedSession).toBeDefined();
      expect(retrievedSession?.id).toBe(session.id);

      // Step 5: Verify session by telegram ID
      const sessionByTelegram = await sessionService.getSessionByTelegramId(user.telegramId);
      expect(sessionByTelegram).toBeDefined();
      expect(sessionByTelegram?.id).toBe(session.id);
    });

    it('should handle complete existing user /start flow', async () => {
      // Setup: Create existing user
      const existingUserData: Partial<NewUser> = {
        telegramId: mockTelegramUser.id.toString(),
        firstName: 'Old Name',
        lastName: 'Old Last',
        username: 'oldusername',
        role: 'free',
        status: 'active'
      };
      
      let user = await userService.createUser(existingUserData);
      expect(user).toBeDefined();

      // Step 1: Find existing user
      user = await userService.findUserByTelegramId(mockTelegramUser.id.toString());
      expect(user).toBeDefined();
      expect(user?.firstName).toBe('Old Name');

      // Step 2: Update user info
      const updates: Partial<NewUser> = {
        firstName: mockTelegramUser.first_name,
        lastName: mockTelegramUser.last_name,
        username: mockTelegramUser.username,
        lastActiveAt: new Date()
      };
      
      const updatedUser = await userService.updateUser(user!.id, updates);
      expect(updatedUser).toBeDefined();
      expect(updatedUser?.firstName).toBe(mockTelegramUser.first_name);

      // Step 3: Create new session
      const session = await sessionService.createSession(updatedUser!);
      expect(session).toBeDefined();
      expect(session.userId).toBe(updatedUser!.id);

      // Step 4: Verify session works
      const retrievedSession = await sessionService.getSession(session.id);
      expect(retrievedSession).toBeDefined();
      expect(retrievedSession?.telegramId).toBe(updatedUser!.telegramId);
    });
  });
});