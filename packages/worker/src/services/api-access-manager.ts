import type {
  UserRoleType,
  SubscriptionTierType,
  ApiAccess,
  RBACOperationResult,
  ExchangeIdType
} from '@celebrum-ai/shared/types';
import type {
  UserRole,
  SubscriptionTier
} from '@celebrum-ai/shared/types';

/**
 * API Access Manager for managing exchange and AI API configurations
 * Handles rate limiting, API key management, and access validation
 */
export class ApiAccessManager {
  private env: any;
  private rateLimitWindows: Map<string, number[]>;

  constructor(env: any) {
    this.env = env;
    this.rateLimitWindows = new Map();
  }

  /**
   * Add exchange API configuration for user
   */
  async addExchangeApi(
    userId: string,
    exchangeId: ExchangeIdType,
    apiKey: string,
    apiSecret: string,
    passphrase?: string,
    sandbox: boolean = false
  ): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'User API access configuration not found',
          timestamp: Date.now(),
          errors: ['API access not initialized']
        };
      }

      // Check if user can add more exchange APIs
      if (apiAccess.limits.maxExchangeApis > 0 && apiAccess.exchangeApis.length >= apiAccess.limits.maxExchangeApis) {
        return {
          success: false,
          message: 'Maximum exchange APIs limit reached',
          timestamp: Date.now(),
          errors: [`Limit: ${apiAccess.limits.maxExchangeApis}, Current: ${apiAccess.exchangeApis.length}`]
        };
      }

      // Check if exchange already exists
      const existingIndex = apiAccess.exchangeApis.findIndex(api => api.exchangeId === exchangeId);
      
      const exchangeApi = {
        exchangeId,
        apiKey: await this.encryptApiKey(apiKey),
        secretKey: await this.encryptApiKey(apiSecret),
        passphrase: passphrase ? await this.encryptApiKey(passphrase) : undefined,
        sandbox,
        permissions: ['read', 'trade'],
        isActive: true,
        lastUsed: Date.now()
      };

      if (existingIndex >= 0) {
        // Update existing
        apiAccess.exchangeApis[existingIndex] = exchangeApi;
      } else {
        // Add new
        apiAccess.exchangeApis.push(exchangeApi);
      }

      apiAccess.lastUpdated = Date.now();

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: `Exchange API ${existingIndex >= 0 ? 'updated' : 'added'} successfully`,
        timestamp: Date.now(),
        data: {
          exchangeId,
          sandbox,
          totalExchangeApis: apiAccess.exchangeApis.length
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to add exchange API',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Add AI API configuration for user
   */
  async addAiApi(
    userId: string,
    provider: string,
    apiKey: string,
    model?: string,
    endpoint?: string
  ): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'User API access configuration not found',
          timestamp: Date.now(),
          errors: ['API access not initialized']
        };
      }

      // Check if user can add more AI APIs
      if (apiAccess.limits.maxAiApis > 0 && apiAccess.aiApis.length >= apiAccess.limits.maxAiApis) {
        return {
          success: false,
          message: 'Maximum AI APIs limit reached',
          timestamp: Date.now(),
          errors: [`Limit: ${apiAccess.limits.maxAiApis}, Current: ${apiAccess.aiApis.length}`]
        };
      }

      // Check if provider already exists
      const existingIndex = apiAccess.aiApis.findIndex(api => api.provider === provider);
      
      const aiApi = {
        provider,
        apiKey: await this.encryptApiKey(apiKey),
        model: model || 'default',
        endpoint: endpoint || '',
        isActive: true,
        addedAt: Date.now(),
        lastUsed: 0,
        requestCount: 0,
        tokensUsed: 0
      };

      if (existingIndex >= 0) {
        // Update existing
        apiAccess.aiApis[existingIndex] = aiApi;
      } else {
        // Add new
        apiAccess.aiApis.push(aiApi);
      }

      apiAccess.lastUpdated = Date.now();

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: `AI API ${existingIndex >= 0 ? 'updated' : 'added'} successfully`,
        timestamp: Date.now(),
        data: {
          provider,
          model,
          totalAiApis: apiAccess.aiApis.length
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to add AI API',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Remove exchange API
   */
  async removeExchangeApi(userId: string, exchangeId: ExchangeIdType): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'User API access configuration not found',
          timestamp: Date.now(),
          errors: ['API access not initialized']
        };
      }

      const initialLength = apiAccess.exchangeApis.length;
      apiAccess.exchangeApis = apiAccess.exchangeApis.filter(api => api.exchangeId !== exchangeId);
      
      if (apiAccess.exchangeApis.length === initialLength) {
        return {
          success: false,
          message: 'Exchange API not found',
          timestamp: Date.now(),
          errors: [`Exchange ${exchangeId} not configured`]
        };
      }

      apiAccess.lastUpdated = Date.now();

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Exchange API removed successfully',
        timestamp: Date.now(),
        data: {
          exchangeId,
          remainingApis: apiAccess.exchangeApis.length
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to remove exchange API',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Remove AI API
   */
  async removeAiApi(userId: string, provider: string): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'User API access configuration not found',
          timestamp: Date.now(),
          errors: ['API access not initialized']
        };
      }

      const initialLength = apiAccess.aiApis.length;
      apiAccess.aiApis = apiAccess.aiApis.filter(api => api.provider !== provider);
      
      if (apiAccess.aiApis.length === initialLength) {
        return {
          success: false,
          message: 'AI API not found',
          timestamp: Date.now(),
          errors: [`Provider ${provider} not configured`]
        };
      }

      apiAccess.lastUpdated = Date.now();

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'AI API removed successfully',
        timestamp: Date.now(),
        data: {
          provider,
          remainingApis: apiAccess.aiApis.length
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to remove AI API',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get decrypted exchange API credentials
   */
  async getExchangeApiCredentials(userId: string, exchangeId: ExchangeIdType): Promise<{
    apiKey: string;
    apiSecret: string;
    passphrase?: string;
    sandbox: boolean;
  } | null> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return null;
      }

      const exchangeApi = apiAccess.exchangeApis.find(api => api.exchangeId === exchangeId && api.isActive);
      if (!exchangeApi) {
        return null;
      }

      // Update last used timestamp
      exchangeApi.lastUsed = Date.now();
      
      // Store updated usage
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        apiKey: await this.decryptApiKey(exchangeApi.apiKey),
        apiSecret: await this.decryptApiKey(exchangeApi.secretKey),
        passphrase: exchangeApi.passphrase ? await this.decryptApiKey(exchangeApi.passphrase) : undefined,
        sandbox: exchangeApi.sandbox
      };
    } catch (error) {
      console.error('Failed to get exchange API credentials:', error);
      return null;
    }
  }

  /**
   * Get decrypted AI API credentials
   */
  async getAiApiCredentials(userId: string, provider: string): Promise<{
    apiKey: string;
    model: string;
    endpoint: string;
  } | null> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return null;
      }

      const aiApi = apiAccess.aiApis.find(api => api.provider === provider && api.isActive);
      if (!aiApi) {
        return null;
      }

      // Update last used timestamp
      aiApi.lastUsed = Date.now();
      
      // Store updated usage
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        apiKey: await this.decryptApiKey(aiApi.apiKey),
        model: aiApi.model || 'default',
        endpoint: `https://api.${provider}.com/v1`
      };
    } catch (error) {
      console.error('Failed to get AI API credentials:', error);
      return null;
    }
  }

  /**
   * Check rate limits for user
   */
  async checkRateLimit(userId: string, window: 'hourly' | 'daily' = 'hourly'): Promise<{
    allowed: boolean;
    remaining: number;
    resetTime: number;
  }> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return { allowed: false, remaining: 0, resetTime: Date.now() };
      }

      const now = Date.now();
      const windowMs = window === 'hourly' ? 60 * 60 * 1000 : 24 * 60 * 60 * 1000;
      const limit = window === 'hourly' ? apiAccess.limits.hourlyRequestLimit : apiAccess.limits.dailyRequestLimit;
      const used = window === 'hourly' ? apiAccess.usage.hourlyRequests : apiAccess.usage.dailyRequests;
      
      // Calculate reset time
      const resetTime = window === 'hourly' 
        ? Math.ceil(now / (60 * 60 * 1000)) * (60 * 60 * 1000)
        : Math.ceil(now / (24 * 60 * 60 * 1000)) * (24 * 60 * 60 * 1000);

      const remaining = Math.max(0, limit - used);
      const allowed = limit === -1 || used < limit; // -1 means unlimited

      return {
        allowed,
        remaining,
        resetTime
      };
    } catch (error) {
      console.error('Failed to check rate limit:', error);
      return { allowed: false, remaining: 0, resetTime: Date.now() };
    }
  }

  /**
   * Get user's API access summary
   */
  async getApiAccessSummary(userId: string): Promise<any | null> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return null;
      }

      // Remove sensitive data for summary
      const summary = {
        userId: apiAccess.userId,
        role: apiAccess.role,
        exchangeApis: apiAccess.exchangeApis.map(api => ({
          exchangeId: api.exchangeId,
          sandbox: api.sandbox,
          isActive: api.isActive,
          lastUsed: api.lastUsed
        })),
        aiApis: apiAccess.aiApis.map(api => ({
          provider: api.provider,
          model: api.model,
          isActive: api.isActive,
          lastUsed: api.lastUsed
        }))
      };

      return summary;
    } catch (error) {
      console.error('Failed to get API access summary:', error);
      return null;
    }
  }

  /**
   * Encrypt API key using Cloudflare's Web Crypto API
   */
  private async encryptApiKey(apiKey: string): Promise<string> {
    try {
      // In production, use proper encryption with a secret key
      // For now, we'll use base64 encoding as a placeholder
      return btoa(apiKey);
    } catch (error) {
      console.error('Failed to encrypt API key:', error);
      return apiKey; // Fallback to plain text (not recommended for production)
    }
  }

  /**
   * Decrypt API key
   */
  private async decryptApiKey(encryptedApiKey: string): Promise<string> {
    try {
      // In production, use proper decryption
      // For now, we'll use base64 decoding as a placeholder
      return atob(encryptedApiKey);
    } catch (error) {
      console.error('Failed to decrypt API key:', error);
      return encryptedApiKey; // Fallback to encrypted text
    }
  }

  /**
   * Toggle API status (active/inactive)
   */
  async toggleApiStatus(userId: string, type: 'exchange' | 'ai', identifier: string, isActive: boolean): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'User API access configuration not found',
          timestamp: Date.now(),
          errors: ['API access not initialized']
        };
      }

      let found = false;
      
      if (type === 'exchange') {
        const api = apiAccess.exchangeApis.find(api => api.exchangeId === identifier);
        if (api) {
          api.isActive = isActive;
          found = true;
        }
      } else {
        const api = apiAccess.aiApis.find(api => api.provider === identifier);
        if (api) {
          api.isActive = isActive;
          found = true;
        }
      }

      if (!found) {
        return {
          success: false,
          message: `${type} API not found`,
          timestamp: Date.now(),
          errors: [`${identifier} not configured`]
        };
      }

      apiAccess.lastUpdated = Date.now();

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: `${type} API ${isActive ? 'activated' : 'deactivated'} successfully`,
        timestamp: Date.now(),
        data: {
          type,
          identifier,
          isActive
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to toggle API status',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }
}