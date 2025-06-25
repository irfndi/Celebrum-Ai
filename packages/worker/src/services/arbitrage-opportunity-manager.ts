import type {
  UserRoleType,
  SubscriptionTierType,
  OpportunityLimits,
  RBACOperationResult,
  Opportunity,
  ArbitrageOpportunity
} from '@celebrum-ai/shared/types';

/**
 * Arbitrage Opportunity Manager for managing opportunity detection and execution limits
 * Handles opportunity validation, rate limiting, and access control
 */
export class ArbitrageOpportunityManager {
  private env: any;
  private opportunityCache: Map<string, ArbitrageOpportunity[]>;
  private userOpportunityHistory: Map<string, any[]>;

  constructor(env: any) {
    this.env = env;
    this.opportunityCache = new Map();
    this.userOpportunityHistory = new Map();
  }

  /**
   * Create default opportunity limits for user
   */
  async createDefaultOpportunityLimits(
    userId: string,
    role: UserRoleType,
    subscriptionTier: SubscriptionTierType
  ): Promise<OpportunityLimits> {
    const limits = this.getTierLimits(subscriptionTier);
    
    const opportunityLimits: OpportunityLimits = {
      dailyLimit: limits.dailyLimit,
      dailyUsed: 0,
      hourlyLimit: limits.hourlyLimit,
      hourlyUsed: 0,
      totalAccessed: 0,
      successRate: 0
    };

    // Store in KV
    const key = `rbac:opportunity_limits:${userId}`;
    await this.env.ArbEdgeKV.put(key, JSON.stringify(opportunityLimits), {
      expirationTtl: 86400 // 24 hours
    });

    return opportunityLimits;
  }

  /**
   * Get opportunity limits for user
   */
  async getOpportunityLimits(userId: string): Promise<OpportunityLimits | null> {
    try {
      const key = `rbac:opportunity_limits:${userId}`;
      const limits = await this.env.ArbEdgeKV.get(key, 'json');
      return limits as OpportunityLimits | null;
    } catch (error) {
      console.error('Failed to get opportunity limits:', error);
      return null;
    }
  }

  /**
   * Validate opportunity access request
   */
  async validateOpportunityAccess(
    userId: string,
    opportunityType: 'view' | 'execute' | 'create_alert'
  ): Promise<RBACOperationResult> {
    try {
      const limits = await this.getOpportunityLimits(userId);
      
      if (!limits) {
        return {
          success: false,
          message: 'Opportunity limits not found',
          timestamp: Date.now(),
          errors: ['User opportunity limits not initialized']
        };
      }

      const now = Date.now();
      
      // Reset counters if needed
      await this.resetCountersIfNeeded(userId, limits, now);

      // Check daily limits
      if (limits.dailyLimit > 0 && limits.dailyUsed >= limits.dailyLimit) {
        return {
          success: false,
          message: 'Daily opportunity limit exceeded',
          timestamp: now,
          errors: [`Daily limit: ${limits.dailyLimit}, Used: ${limits.dailyUsed}`]
        };
      }

      // Check hourly limits
      if (limits.hourlyLimit > 0 && limits.hourlyUsed >= limits.hourlyLimit) {
        return {
          success: false,
          message: 'Hourly opportunity limit exceeded',
          timestamp: now,
          errors: [`Hourly limit: ${limits.hourlyLimit}, Used: ${limits.hourlyUsed}`]
        };
      }

      // Update usage counters for non-view operations
      if (opportunityType !== 'view') {
        limits.dailyUsed++;
        limits.hourlyUsed++;
        limits.totalAccessed++;

        // Store updated limits
        const key = `rbac:opportunity_limits:${userId}`;
        await this.env.ArbEdgeKV.put(key, JSON.stringify(limits), {
          expirationTtl: 86400
        });
      }

      return {
        success: true,
        message: 'Opportunity access validated successfully',
        timestamp: now,
        data: {
          remainingDaily: Math.max(0, limits.dailyLimit - limits.dailyUsed),
          remainingHourly: Math.max(0, limits.hourlyLimit - limits.hourlyUsed),
          totalAccessed: limits.totalAccessed,
          successRate: limits.successRate
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to validate opportunity access',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Record opportunity execution result
   */
  async recordOpportunityExecution(
    userId: string,
    opportunityId: string,
    success: boolean,
    profitLoss?: number,
    executionTime?: number
  ): Promise<RBACOperationResult> {
    try {
      const limits = await this.getOpportunityLimits(userId);
      
      if (!limits) {
        return {
          success: false,
          message: 'Opportunity limits not found',
          timestamp: Date.now(),
          errors: ['User opportunity limits not initialized']
        };
      }

      // Update success rate
      const totalExecutions = await this.getTotalExecutions(userId);
      const successfulExecutions = await this.getSuccessfulExecutions(userId);
      
      if (success) {
        limits.successRate = ((successfulExecutions + 1) / (totalExecutions + 1)) * 100;
      } else {
        limits.successRate = (successfulExecutions / (totalExecutions + 1)) * 100;
      }

      // Store execution record
      const executionRecord = {
        opportunityId,
        userId,
        success,
        profitLoss: profitLoss || 0,
        executionTime: executionTime || Date.now(),
        timestamp: Date.now()
      };

      const executionKey = `rbac:opportunity_execution:${userId}:${Date.now()}`;
      await this.env.ArbEdgeKV.put(executionKey, JSON.stringify(executionRecord), {
        expirationTtl: 7 * 24 * 60 * 60 // 7 days
      });

      // Update limits
      const limitsKey = `rbac:opportunity_limits:${userId}`;
      await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Opportunity execution recorded successfully',
        timestamp: Date.now(),
        data: {
          opportunityId,
          executionSuccess: success,
          newSuccessRate: limits.successRate,
          profitLoss
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to record opportunity execution',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get available opportunities for user based on their access level
   */
  async getAvailableOpportunities(
    userId: string,
    role: UserRoleType,
    filters?: {
      minProfitPercent?: number;
      maxRisk?: 'low' | 'medium' | 'high';
      exchanges?: string[];
      symbols?: string[];
    }
  ): Promise<ArbitrageOpportunity[]> {
    try {
      // Validate access first
      const accessValidation = await this.validateOpportunityAccess(userId, 'view');
      if (!accessValidation.success) {
        return [];
      }

      // Get opportunities from cache or generate new ones
      let opportunities = await this.getOpportunitiesFromCache(role);
      
      if (opportunities.length === 0) {
        opportunities = await this.generateOpportunities(role);
        await this.cacheOpportunities(role, opportunities);
      }

      // Apply filters
      if (filters) {
        opportunities = this.applyFilters(opportunities, filters);
      }

      // Limit based on role
      const maxOpportunities = this.getMaxOpportunitiesForRole(role);
      opportunities = opportunities.slice(0, maxOpportunities);

      return opportunities;
    } catch (error) {
      console.error('Failed to get available opportunities:', error);
      return [];
    }
  }

  /**
   * Create opportunity alert for user
   */
  async createOpportunityAlert(
    userId: string,
    alertConfig: {
      minProfitPercent: number;
      maxRisk: 'low' | 'medium' | 'high';
      exchanges: string[];
      symbols?: string[];
      notificationMethod: 'email' | 'webhook' | 'telegram';
      isActive: boolean;
    }
  ): Promise<RBACOperationResult> {
    try {
      // Validate access
      const accessValidation = await this.validateOpportunityAccess(userId, 'create_alert');
      if (!accessValidation.success) {
        return accessValidation;
      }

      const alertId = `alert_${userId}_${Date.now()}`;
      const alert = {
        id: alertId,
        userId,
        ...alertConfig,
        createdAt: Date.now(),
        lastTriggered: 0,
        triggerCount: 0
      };

      // Store alert
      const alertKey = `rbac:opportunity_alert:${alertId}`;
      await this.env.ArbEdgeKV.put(alertKey, JSON.stringify(alert), {
        expirationTtl: 30 * 24 * 60 * 60 // 30 days
      });

      // Add to user's alert list
      const userAlertsKey = `rbac:user_alerts:${userId}`;
      const existingAlerts = await this.env.ArbEdgeKV.get(userAlertsKey, 'json') || [];
      existingAlerts.push(alertId);
      
      await this.env.ArbEdgeKV.put(userAlertsKey, JSON.stringify(existingAlerts), {
        expirationTtl: 30 * 24 * 60 * 60
      });

      return {
        success: true,
        message: 'Opportunity alert created successfully',
        timestamp: Date.now(),
        data: {
          alertId,
          config: alertConfig
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to create opportunity alert',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get user's opportunity alerts
   */
  async getUserOpportunityAlerts(userId: string): Promise<any[]> {
    try {
      const userAlertsKey = `rbac:user_alerts:${userId}`;
      const alertIds = await this.env.ArbEdgeKV.get(userAlertsKey, 'json') || [];
      
      const alerts = [];
      for (const alertId of alertIds) {
        const alertKey = `rbac:opportunity_alert:${alertId}`;
        const alert = await this.env.ArbEdgeKV.get(alertKey, 'json');
        if (alert) {
          alerts.push(alert);
        }
      }
      
      return alerts;
    } catch (error) {
      console.error('Failed to get user opportunity alerts:', error);
      return [];
    }
  }

  /**
   * Update opportunity limits for user
   */
  async updateOpportunityLimits(
    userId: string,
    newLimits: Partial<OpportunityLimits>
  ): Promise<RBACOperationResult> {
    try {
      const existingLimits = await this.getOpportunityLimits(userId);
      
      if (!existingLimits) {
        return {
          success: false,
          message: 'Opportunity limits not found',
          timestamp: Date.now(),
          errors: ['User opportunity limits not initialized']
        };
      }

      const updatedLimits: OpportunityLimits = {
        ...existingLimits,
        ...newLimits
      };

      // Store updated limits
      const key = `rbac:opportunity_limits:${userId}`;
      await this.env.ArbEdgeKV.put(key, JSON.stringify(updatedLimits), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Opportunity limits updated successfully',
        timestamp: Date.now(),
        data: updatedLimits
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update opportunity limits',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get tier-based opportunity limits
   */
  private getTierLimits(tier: SubscriptionTierType): {
    dailyLimit: number;
    hourlyLimit: number;
  } {
    switch (tier) {
      case 'free':
        return { dailyLimit: 10, hourlyLimit: 5 };
      case 'pro':
        return { dailyLimit: 100, hourlyLimit: 25 };
      case 'ultra':
        return { dailyLimit: 1000, hourlyLimit: 100 };
      case 'enterprise':
        return { dailyLimit: -1, hourlyLimit: -1 }; // unlimited
      default:
        return { dailyLimit: 10, hourlyLimit: 5 };
    }
  }

  /**
   * Reset counters if time windows have passed
   */
  private async resetCountersIfNeeded(
    userId: string,
    limits: OpportunityLimits,
    currentTime: number
  ): Promise<void> {
    try {
      const lastResetKey = `rbac:opportunity_reset:${userId}`;
      const lastReset = await this.env.ArbEdgeKV.get(lastResetKey, 'json') || {
        dailyReset: 0,
        hourlyReset: 0
      };

      const now = new Date(currentTime);
      const lastDailyReset = new Date(lastReset.dailyReset);
      const lastHourlyReset = new Date(lastReset.hourlyReset);

      let needsUpdate = false;

      // Check if we need to reset daily counter
      if (now.getDate() !== lastDailyReset.getDate() || 
          now.getMonth() !== lastDailyReset.getMonth() || 
          now.getFullYear() !== lastDailyReset.getFullYear()) {
        limits.dailyUsed = 0;
        lastReset.dailyReset = currentTime;
        needsUpdate = true;
      }

      // Check if we need to reset hourly counter
      if (now.getHours() !== lastHourlyReset.getHours() || 
          now.getDate() !== lastHourlyReset.getDate()) {
        limits.hourlyUsed = 0;
        lastReset.hourlyReset = currentTime;
        needsUpdate = true;
      }

      if (needsUpdate) {
        // Update reset timestamps
        await this.env.ArbEdgeKV.put(lastResetKey, JSON.stringify(lastReset), {
          expirationTtl: 86400
        });

        // Update limits
        const limitsKey = `rbac:opportunity_limits:${userId}`;
        await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
          expirationTtl: 86400
        });
      }
    } catch (error) {
      console.error('Failed to reset counters:', error);
    }
  }

  /**
   * Get opportunities from cache
   */
  private async getOpportunitiesFromCache(role: UserRoleType): Promise<ArbitrageOpportunity[]> {
    try {
      const cacheKey = `opportunities_cache:${role}`;
      const cached = await this.env.ArbEdgeKV.get(cacheKey, 'json');
      return cached ? cached.opportunities : [];
    } catch (error) {
      console.error('Failed to get opportunities from cache:', error);
      return [];
    }
  }

  /**
   * Cache opportunities for role
   */
  private async cacheOpportunities(role: UserRoleType, opportunities: ArbitrageOpportunity[]): Promise<void> {
    try {
      const cacheKey = `opportunities_cache:${role}`;
      const cacheData = {
        opportunities,
        timestamp: Date.now()
      };
      
      await this.env.ArbEdgeKV.put(cacheKey, JSON.stringify(cacheData), {
        expirationTtl: 300 // 5 minutes
      });
    } catch (error) {
      console.error('Failed to cache opportunities:', error);
    }
  }

  /**
   * Generate mock opportunities (in production, this would connect to real data sources)
   */
  private async generateOpportunities(role: UserRoleType): Promise<ArbitrageOpportunity[]> {
    // Mock opportunity generation - in production, this would fetch real data
    const baseOpportunities: ArbitrageOpportunity[] = [
      {
        id: `opp_${Date.now()}_1`,
        symbol: 'BTC/USDT',
        exchange_a: 'binance',
        exchange_b: 'coinbase',
        price_a: 45000,
        price_b: 45200,
        profit_percentage: 0.44,
        confidence_score: 0.95,
        generated_at: new Date().toISOString(),
        expires_at: new Date(Date.now() + 300000).toISOString()
      },
      {
        id: `opp_${Date.now()}_2`,
        symbol: 'ETH/USDT',
        exchange_a: 'kraken',
        exchange_b: 'binance',
        price_a: 3200,
        price_b: 3220,
        profit_percentage: 0.625,
        confidence_score: 0.85,
        generated_at: new Date().toISOString(),
        expires_at: new Date(Date.now() + 300000).toISOString()
      }
    ];

    // Filter opportunities based on role
    return this.filterOpportunitiesByRole(baseOpportunities, role);
  }

  /**
   * Filter opportunities by user role
   */
  private filterOpportunitiesByRole(opportunities: ArbitrageOpportunity[], role: UserRoleType): ArbitrageOpportunity[] {
    switch (role) {
      case 'free':
        return opportunities.filter(opp => opp.confidence_score >= 0.8 && opp.profit_percentage >= 0.3);
      case 'pro':
        return opportunities.filter(opp => opp.confidence_score >= 0.7 && opp.profit_percentage >= 0.2);
      case 'ultra':
      case 'admin':
      case 'superadmin':
        return opportunities; // All opportunities
      default:
        return opportunities.filter(opp => opp.confidence_score >= 0.8);
    }
  }

  /**
   * Apply user-defined filters to opportunities
   */
  private applyFilters(opportunities: ArbitrageOpportunity[], filters: any): ArbitrageOpportunity[] {
    return opportunities.filter(opp => {
      if (filters.minProfitPercent && opp.profit_percentage < filters.minProfitPercent) {
        return false;
      }
      
      // Risk filtering removed as ArbitrageOpportunity schema doesn't include risk property
      // Risk can be calculated from confidence_score if needed
      
      if (filters.exchanges && filters.exchanges.length > 0) {
        if (!filters.exchanges.includes(opp.exchange_a) && !filters.exchanges.includes(opp.exchange_b)) {
          return false;
        }
      }
      
      if (filters.symbols && filters.symbols.length > 0) {
        if (!filters.symbols.includes(opp.symbol)) {
          return false;
        }
      }
      
      return true;
    });
  }

  /**
   * Get maximum opportunities for role
   */
  private getMaxOpportunitiesForRole(role: UserRoleType): number {
    switch (role) {
      case 'free': return 5;
      case 'pro': return 20;
      case 'ultra': return 50;
      case 'admin':
      case 'superadmin': return 100;
      default: return 5;
    }
  }

  /**
   * Get total executions for user
   */
  private async getTotalExecutions(userId: string): Promise<number> {
    try {
      const statsKey = `rbac:opportunity_stats:${userId}`;
      const stats = await this.env.ArbEdgeKV.get(statsKey, 'json') || { total: 0, successful: 0 };
      return stats.total;
    } catch (error) {
      return 0;
    }
  }

  /**
   * Get successful executions for user
   */
  private async getSuccessfulExecutions(userId: string): Promise<number> {
    try {
      const statsKey = `rbac:opportunity_stats:${userId}`;
      const stats = await this.env.ArbEdgeKV.get(statsKey, 'json') || { total: 0, successful: 0 };
      return stats.successful;
    } catch (error) {
      return 0;
    }
  }
}