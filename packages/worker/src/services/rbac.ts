import type {
  UserRoleType,
  SubscriptionTierType,
  PermissionType,
  TradingConfig,
  ApiAccess,
  UserAccessSummary,
  OpportunityLimits,
  StrategyLimits,
  RBACOperationResult,
  TechnicalStrategy,
  RiskManagementConfig,
  RiskLevelType
} from '@celebrum-ai/shared/types';
import {
  Permission,
  UserRole,
  SubscriptionTier,
  RiskLevel
} from '@celebrum-ai/shared/types';

/**
 * Comprehensive RBAC Service for ArbEdge
 * Manages user roles, permissions, API access, trading configurations, and feature flags
 */
export class RBACService {
  private env: any;
  private rolePermissions: Map<UserRoleType, PermissionType[]>;
  private tierLimits: Map<SubscriptionTierType, any>;
  private featureFlags: Map<string, boolean>;

  constructor(env: any) {
    this.env = env;
    this.rolePermissions = new Map();
    this.tierLimits = new Map();
    this.featureFlags = new Map();
    this.initializeRolePermissions();
    this.initializeTierLimits();
    this.initializeFeatureFlags();
  }

  /**
   * Initialize role-based permissions matrix
   */
  private initializeRolePermissions(): void {
    this.rolePermissions.set(UserRole.FREE, [
      Permission.READ_PROFILE,
      Permission.UPDATE_PROFILE,
      Permission.TRADE_VIEW_POSITIONS,
      Permission.OPPORTUNITY_VIEW,
      Permission.STRATEGY_VIEW
    ]);

    this.rolePermissions.set(UserRole.PRO, [
      ...this.rolePermissions.get(UserRole.FREE)!,
      Permission.TRADE_MANUAL,
      Permission.API_EXCHANGE_ACCESS,
      Permission.OPPORTUNITY_EXECUTE,
      Permission.STRATEGY_CREATE,
      Permission.STRATEGY_BACKTEST
    ]);

    this.rolePermissions.set(UserRole.ULTRA, [
      ...this.rolePermissions.get(UserRole.PRO)!,
      Permission.TRADE_AUTO,
      Permission.TRADE_MANAGE_CONFIG,
      Permission.API_AI_ACCESS,
      Permission.API_MANAGE_KEYS,
      Permission.OPPORTUNITY_CREATE_ALERTS,
      Permission.STRATEGY_EXECUTE
    ]);

    this.rolePermissions.set(UserRole.ADMIN, [
      ...this.rolePermissions.get(UserRole.ULTRA)!,
      Permission.ADMIN_USER_MANAGEMENT,
      Permission.ADMIN_VIEW_ANALYTICS,
      Permission.ADMIN_MANAGE_FEATURES
    ]);

    this.rolePermissions.set(UserRole.SUPERADMIN, [
      ...this.rolePermissions.get(UserRole.ADMIN)!,
      Permission.ADMIN_SYSTEM_CONFIG,
      Permission.SUPERADMIN_FULL_ACCESS
    ]);
  }

  /**
   * Initialize subscription tier limits
   */
  private initializeTierLimits(): void {
    this.tierLimits.set(SubscriptionTier.FREE, {
      maxExchangeApis: 1,
      maxAiApis: 0,
      dailyRequestLimit: 100,
      hourlyRequestLimit: 20,
      maxConcurrentTrades: 1,
      maxLeverage: 3,
      maxStrategies: 1,
      maxActiveStrategies: 0,
      maxConcurrentBacktests: 1,
      dailyOpportunityLimit: 10,
      hourlyOpportunityLimit: 5
    });

    this.tierLimits.set(SubscriptionTier.PRO, {
      maxExchangeApis: 3,
      maxAiApis: 1,
      dailyRequestLimit: 1000,
      hourlyRequestLimit: 200,
      maxConcurrentTrades: 5,
      maxLeverage: 10,
      maxStrategies: 5,
      maxActiveStrategies: 2,
      maxConcurrentBacktests: 3,
      dailyOpportunityLimit: 100,
      hourlyOpportunityLimit: 25
    });

    this.tierLimits.set(SubscriptionTier.ULTRA, {
      maxExchangeApis: 10,
      maxAiApis: 3,
      dailyRequestLimit: 10000,
      hourlyRequestLimit: 1000,
      maxConcurrentTrades: 20,
      maxLeverage: 50,
      maxStrategies: 20,
      maxActiveStrategies: 10,
      maxConcurrentBacktests: 10,
      dailyOpportunityLimit: 1000,
      hourlyOpportunityLimit: 100
    });

    this.tierLimits.set(SubscriptionTier.ENTERPRISE, {
      maxExchangeApis: -1, // unlimited
      maxAiApis: -1,
      dailyRequestLimit: -1,
      hourlyRequestLimit: -1,
      maxConcurrentTrades: 100,
      maxLeverage: 100,
      maxStrategies: -1,
      maxActiveStrategies: -1,
      maxConcurrentBacktests: -1,
      dailyOpportunityLimit: -1,
      hourlyOpportunityLimit: -1
    });
  }

  /**
   * Initialize feature flags
   */
  private initializeFeatureFlags(): void {
    this.featureFlags.set('rbac.enabled', true);
    this.featureFlags.set('api_access.enabled', true);
    this.featureFlags.set('trading.enabled', true);
    this.featureFlags.set('opportunity_engine.enabled', true);
    this.featureFlags.set('technical_strategies.enabled', true);
    this.featureFlags.set('auto_trading.enabled', true);
    this.featureFlags.set('advanced_analytics.enabled', true);
    this.featureFlags.set('yaml_strategies.enabled', true);
  }

  /**
   * Check if user has specific permission
   */
  hasPermission(role: UserRoleType, permission: PermissionType): boolean {
    const permissions = this.rolePermissions.get(role);
    return permissions ? permissions.includes(permission) : false;
  }

  /**
   * Get all permissions for a role
   */
  getRolePermissions(role: UserRoleType): PermissionType[] {
    return this.rolePermissions.get(role) || [];
  }

  /**
   * Get tier limits for subscription
   */
  getTierLimits(tier: SubscriptionTierType): any {
    return this.tierLimits.get(tier) || this.tierLimits.get(SubscriptionTier.FREE);
  }

  /**
   * Check if feature flag is enabled
   */
  isFeatureEnabled(flag: string): boolean {
    return this.featureFlags.get(flag) || false;
  }

  /**
   * Create comprehensive user access summary
   */
  async createUserAccessSummary(
    userId: string,
    role: UserRoleType,
    subscriptionTier: SubscriptionTierType
  ): Promise<UserAccessSummary> {
    const permissions = this.getRolePermissions(role);
    const limits = this.getTierLimits(subscriptionTier);
    const timestamp = Date.now();

    // Get or create API access configuration
    const apiAccess: ApiAccess = await this.getOrCreateApiAccess(userId, role, limits);

    // Get or create trading configuration
    const tradingConfig: TradingConfig | undefined = await this.getOrCreateTradingConfig(userId, role, limits);

    // Create opportunity limits
    const opportunityLimits: OpportunityLimits = {
      dailyLimit: limits.dailyOpportunityLimit,
      dailyUsed: 0,
      hourlyLimit: limits.hourlyOpportunityLimit,
      hourlyUsed: 0,
      totalAccessed: 0,
      successRate: 0
    };

    // Create strategy limits
    const strategyLimits: StrategyLimits = {
      maxStrategies: limits.maxStrategies,
      createdStrategies: 0,
      maxActiveStrategies: limits.maxActiveStrategies,
      activeStrategies: 0,
      maxConcurrentBacktests: limits.maxConcurrentBacktests,
      concurrentBacktests: 0
    };

    // Get feature flags as object
    const featureFlags: Record<string, boolean> = {};
    this.featureFlags.forEach((value, key) => {
      featureFlags[key] = value;
    });

    return {
      userId,
      role,
      subscriptionTier,
      permissions,
      apiAccess,
      tradingConfig,
      opportunityLimits,
      strategyLimits,
      featureFlags,
      lastUpdated: timestamp
    };
  }

  /**
   * Get or create API access configuration
   */
  private async getOrCreateApiAccess(userId: string, role: UserRoleType, limits: any): Promise<ApiAccess> {
    const key = `rbac:api_access:${userId}`;
    
    try {
      const existing = await this.env.ArbEdgeKV.get(key, 'json');
      if (existing) {
        return existing as ApiAccess;
      }
    } catch (error) {
      console.warn('Failed to get existing API access:', error);
    }

    // Create new API access configuration
    const apiAccess: ApiAccess = {
      userId,
      role,
      exchangeApis: [],
      aiApis: [],
      limits: {
        maxExchangeApis: limits.maxExchangeApis,
        maxAiApis: limits.maxAiApis,
        dailyRequestLimit: limits.dailyRequestLimit,
        hourlyRequestLimit: limits.hourlyRequestLimit
      },
      usage: {
        dailyRequests: 0,
        hourlyRequests: 0,
        totalRequests: 0,
        lastReset: Date.now()
      },
      lastUpdated: Date.now()
    };

    // Store in KV
    await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
      expirationTtl: 86400 // 24 hours
    });

    return apiAccess;
  }

  /**
   * Get or create trading configuration
   */
  private async getOrCreateTradingConfig(userId: string, role: UserRoleType, limits: any): Promise<TradingConfig | undefined> {
    // Only create trading config for roles that can trade
    if (!this.hasPermission(role, Permission.TRADE_MANUAL) && !this.hasPermission(role, Permission.TRADE_AUTO)) {
      return undefined;
    }

    const key = `rbac:trading_config:${userId}`;
    
    try {
      const existing = await this.env.ArbEdgeKV.get(key, 'json');
      if (existing) {
        return existing as TradingConfig;
      }
    } catch (error) {
      console.warn('Failed to get existing trading config:', error);
    }

    // Create default risk management config based on role
    const riskManagement: RiskManagementConfig = {
      maxDailyLossPercent: role === UserRole.FREE ? 5 : role === UserRole.PRO ? 10 : 20,
      maxDrawdownPercent: role === UserRole.FREE ? 10 : role === UserRole.PRO ? 15 : 25,
      positionSizingMethod: 'percentage_of_portfolio',
      stopLossRequired: role === UserRole.FREE,
      takeProfitRecommended: true,
      trailingStopEnabled: role !== UserRole.FREE,
      riskRewardRatioMin: 1.5
    };

    // Create new trading configuration
    const tradingConfig: TradingConfig = {
      userId,
      role,
      percentagePerTrade: role === UserRole.FREE ? 2 : role === UserRole.PRO ? 5 : 10,
      maxConcurrentTrades: limits.maxConcurrentTrades,
      maxLeverage: limits.maxLeverage,
      riskTolerance: 'medium',
      autoTradingEnabled: this.hasPermission(role, Permission.TRADE_AUTO),
      manualTradingEnabled: this.hasPermission(role, Permission.TRADE_MANUAL),
      riskManagement,
      lastUpdated: Date.now()
    };

    // Store in KV
    await this.env.ArbEdgeKV.put(key, JSON.stringify(tradingConfig), {
      expirationTtl: 86400 // 24 hours
    });

    return tradingConfig;
  }

  /**
   * Validate API access request
   */
  async validateApiAccess(userId: string, apiType: 'exchange' | 'ai'): Promise<RBACOperationResult> {
    try {
      const key = `rbac:api_access:${userId}`;
      const apiAccess = await this.env.ArbEdgeKV.get(key, 'json') as ApiAccess;
      
      if (!apiAccess) {
        return {
          success: false,
          message: 'API access configuration not found',
          timestamp: Date.now(),
          errors: ['User API access not configured']
        };
      }

      // Check daily limits
      const now = Date.now();
      const daysSinceReset = Math.floor((now - apiAccess.usage.lastReset) / (24 * 60 * 60 * 1000));
      
      if (daysSinceReset >= 1) {
        // Reset daily counters
        apiAccess.usage.dailyRequests = 0;
        apiAccess.usage.lastReset = now;
      }

      // Check hourly limits
      const hoursSinceReset = Math.floor((now - apiAccess.usage.lastReset) / (60 * 60 * 1000));
      if (hoursSinceReset >= 1) {
        apiAccess.usage.hourlyRequests = 0;
      }

      // Validate limits
      if (apiAccess.limits.dailyRequestLimit > 0 && apiAccess.usage.dailyRequests >= apiAccess.limits.dailyRequestLimit) {
        return {
          success: false,
          message: 'Daily API request limit exceeded',
          timestamp: now,
          errors: ['Daily limit exceeded']
        };
      }

      if (apiAccess.limits.hourlyRequestLimit > 0 && apiAccess.usage.hourlyRequests >= apiAccess.limits.hourlyRequestLimit) {
        return {
          success: false,
          message: 'Hourly API request limit exceeded',
          timestamp: now,
          errors: ['Hourly limit exceeded']
        };
      }

      // Check API type specific limits
      if (apiType === 'exchange' && apiAccess.limits.maxExchangeApis > 0 && apiAccess.exchangeApis.length >= apiAccess.limits.maxExchangeApis) {
        return {
          success: false,
          message: 'Maximum exchange APIs limit reached',
          timestamp: now,
          errors: ['Exchange API limit exceeded']
        };
      }

      if (apiType === 'ai' && apiAccess.limits.maxAiApis > 0 && apiAccess.aiApis.length >= apiAccess.limits.maxAiApis) {
        return {
          success: false,
          message: 'Maximum AI APIs limit reached',
          timestamp: now,
          errors: ['AI API limit exceeded']
        };
      }

      // Update usage counters
      apiAccess.usage.dailyRequests++;
      apiAccess.usage.hourlyRequests++;
      apiAccess.usage.totalRequests++;
      apiAccess.lastUpdated = now;

      // Store updated usage
      await this.env.ArbEdgeKV.put(key, JSON.stringify(apiAccess), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'API access validated successfully',
        timestamp: now,
        data: {
          remainingDaily: apiAccess.limits.dailyRequestLimit - apiAccess.usage.dailyRequests,
          remainingHourly: apiAccess.limits.hourlyRequestLimit - apiAccess.usage.hourlyRequests
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to validate API access',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Validate trading request
   */
  async validateTradingRequest(userId: string, requestedLeverage: number, positionSize: number): Promise<RBACOperationResult> {
    try {
      const key = `rbac:trading_config:${userId}`;
      const tradingConfig = await this.env.ArbEdgeKV.get(key, 'json') as TradingConfig;
      
      if (!tradingConfig) {
        return {
          success: false,
          message: 'Trading configuration not found',
          timestamp: Date.now(),
          errors: ['User trading configuration not found']
        };
      }

      const errors: string[] = [];

      // Validate leverage
      if (requestedLeverage > tradingConfig.maxLeverage) {
        errors.push(`Leverage ${requestedLeverage} exceeds maximum allowed ${tradingConfig.maxLeverage}`);
      }

      // Validate position size
      if (positionSize > tradingConfig.percentagePerTrade) {
        errors.push(`Position size ${positionSize}% exceeds maximum allowed ${tradingConfig.percentagePerTrade}%`);
      }

      // Check if trading is enabled
      if (!tradingConfig.manualTradingEnabled && !tradingConfig.autoTradingEnabled) {
        errors.push('Trading is disabled for this user');
      }

      if (errors.length > 0) {
        return {
          success: false,
          message: 'Trading request validation failed',
          timestamp: Date.now(),
          errors
        };
      }

      return {
        success: true,
        message: 'Trading request validated successfully',
        timestamp: Date.now(),
        data: {
          maxLeverage: tradingConfig.maxLeverage,
          maxPositionSize: tradingConfig.percentagePerTrade,
          riskManagement: tradingConfig.riskManagement
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to validate trading request',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Register a new user with initial role and subscription tier
   */
  async registerUser(userId: string, role: UserRoleType = UserRole.FREE, subscriptionTier: SubscriptionTierType = SubscriptionTier.FREE): Promise<RBACOperationResult> {
    try {
      // Check if user already exists
      const existingUser = await this.getUserAccessSummary(userId);
      if (existingUser) {
        return {
          success: false,
          message: 'User already exists',
          timestamp: Date.now(),
          errors: ['User is already registered']
        };
      }

      // Create new user access summary
      const accessSummary = await this.createUserAccessSummary(userId, role, subscriptionTier);
      
      // Store user summary
      const summaryKey = `rbac:user_summary:${userId}`;
      await this.env.ArbEdgeKV.put(summaryKey, JSON.stringify(accessSummary), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: `User registered successfully with ${role} role and ${subscriptionTier} tier`,
        timestamp: Date.now(),
        data: accessSummary
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to register user',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Update user role and recalculate permissions
   */
  async updateUserRole(userId: string, newRole: UserRoleType, newTier: SubscriptionTierType): Promise<RBACOperationResult> {
    try {
      // Create new access summary
      const accessSummary = await this.createUserAccessSummary(userId, newRole, newTier);
      
      // Store updated summary
      const summaryKey = `rbac:user_summary:${userId}`;
      await this.env.ArbEdgeKV.put(summaryKey, JSON.stringify(accessSummary), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: `User role updated to ${newRole} with ${newTier} tier`,
        timestamp: Date.now(),
        data: accessSummary
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update user role',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get user access summary
   */
  async getUserAccessSummary(userId: string): Promise<UserAccessSummary | null> {
    try {
      const key = `rbac:user_summary:${userId}`;
      const summary = await this.env.ArbEdgeKV.get(key, 'json');
      return summary as UserAccessSummary | null;
    } catch (error) {
      console.error('Failed to get user access summary:', error);
      return null;
    }
  }
}