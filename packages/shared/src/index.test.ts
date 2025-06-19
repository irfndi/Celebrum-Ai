import { describe, it, expect } from 'vitest';
import { UserSchema } from './index';
import type { ApiResponse } from './index';

describe('Shared Package', () => {
  it('should export UserSchema', () => {
    expect(UserSchema).toBeDefined();
    expect(typeof UserSchema.parse).toBe('function');
  });

  it('should validate UserSchema with valid data', () => {
    const validUser = {
      id: 123,
      telegramId: '456789',
      username: 'testuser',
      role: 'free' as const,
      createdAt: new Date('2024-01-01T00:00:00Z'),
      updatedAt: new Date('2024-01-01T00:00:00Z')
    };
    
    const result = UserSchema.safeParse(validUser);
    expect(result.success).toBe(true);
  });

  it('should reject invalid UserSchema data', () => {
    const invalidUser = {
      id: '123', // should be number
      role: 'invalid', // invalid role
    };
    
    const result = UserSchema.safeParse(invalidUser);
    expect(result.success).toBe(false);
  });

  it('should work with ApiResponse type', () => {
    const response: ApiResponse<string> = {
      success: true,
      data: 'test data',
      timestamp: '2024-01-01T00:00:00Z'
    };
    
    expect(response.success).toBe(true);
    expect(response.data).toBe('test data');
  });
});