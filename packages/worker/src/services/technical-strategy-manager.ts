import type {
  UserRoleType,
  SubscriptionTierType,
  StrategyLimits,
  RBACOperationResult,
  TechnicalStrategy
} from '@celebrum-ai/shared/types';

/**
 * Technical Strategy Manager for YAML-based strategy system
 * Handles strategy creation, validation, backtesting, and execution limits
 */
export class TechnicalStrategyManager {
  private env: any;
  private strategyCache: Map<string, TechnicalStrategy[]>;
  private backtestQueue: Map<string, any[]>;

  constructor(env: any) {
    this.env = env;
    this.strategyCache = new Map();
    this.backtestQueue = new Map();
  }

  /**
   * Create default strategy limits for user
   */
  async createDefaultStrategyLimits(
    userId: string,
    role: UserRoleType,
    subscriptionTier: SubscriptionTierType
  ): Promise<StrategyLimits> {
    const limits = this.getTierLimits(subscriptionTier);
    
    const strategyLimits: StrategyLimits = {
      maxStrategies: limits.maxStrategies,
      createdStrategies: 0,
      activeStrategies: 0,
      maxConcurrentBacktests: limits.maxConcurrentBacktests,
      runningBacktests: 0
    };

    // Store in KV
    const key = `rbac:strategy_limits:${userId}`;
    await this.env.ArbEdgeKV.put(key, JSON.stringify(strategyLimits), {
      expirationTtl: 86400 // 24 hours
    });

    return strategyLimits;
  }

  /**
   * Get strategy limits for user
   */
  async getStrategyLimits(userId: string): Promise<StrategyLimits | null> {
    try {
      const key = `rbac:strategy_limits:${userId}`;
      const limits = await this.env.ArbEdgeKV.get(key, 'json');
      return limits as StrategyLimits | null;
    } catch (error) {
      console.error('Failed to get strategy limits:', error);
      return null;
    }
  }

  /**
   * Validate strategy operation
   */
  async validateStrategyOperation(
    userId: string,
    operation: 'create' | 'activate' | 'backtest' | 'delete'
  ): Promise<RBACOperationResult> {
    try {
      const limits = await this.getStrategyLimits(userId);
      
      if (!limits) {
        return {
          success: false,
          message: 'Strategy limits not found',
          timestamp: Date.now(),
          errors: ['User strategy limits not initialized']
        };
      }

      const now = Date.now();

      switch (operation) {
        case 'create':
          if (limits.maxStrategies > 0 && limits.createdStrategies >= limits.maxStrategies) {
            return {
              success: false,
              message: 'Maximum strategies limit exceeded',
              timestamp: now,
              errors: [`Max strategies: ${limits.maxStrategies}, Created: ${limits.createdStrategies}`]
            };
          }
          break;

        case 'activate':
          // Check if we can activate more strategies (usually same as max strategies)
          if (limits.maxStrategies > 0 && limits.activeStrategies >= limits.maxStrategies) {
            return {
              success: false,
              message: 'Maximum active strategies limit exceeded',
              timestamp: now,
              errors: [`Max active: ${limits.maxStrategies}, Active: ${limits.activeStrategies}`]
            };
          }
          break;

        case 'backtest':
          if (limits.maxConcurrentBacktests > 0 && limits.runningBacktests >= limits.maxConcurrentBacktests) {
            return {
              success: false,
              message: 'Maximum concurrent backtests limit exceeded',
              timestamp: now,
              errors: [`Max backtests: ${limits.maxConcurrentBacktests}, Running: ${limits.runningBacktests}`]
            };
          }
          break;

        case 'delete':
          // Delete operations are generally allowed
          break;
      }

      return {
        success: true,
        message: `Strategy ${operation} operation validated successfully`,
        timestamp: now,
        data: {
          remainingStrategies: Math.max(0, limits.maxStrategies - limits.createdStrategies),
          remainingBacktests: Math.max(0, limits.maxConcurrentBacktests - limits.runningBacktests),
          activeStrategies: limits.activeStrategies
        }
      };
    } catch (error) {
      return {
        success: false,
        message: `Failed to validate strategy ${operation}`,
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Create a new technical strategy from YAML configuration
   */
  async createStrategy(
    userId: string,
    strategyConfig: {
      name: string;
      description: string;
      yamlConfig: string;
      symbols: string[];
      timeframes: string[];
      riskLevel: 'low' | 'medium' | 'high';
      isActive: boolean;
    }
  ): Promise<RBACOperationResult> {
    try {
      // Validate operation
      const validation = await this.validateStrategyOperation(userId, 'create');
      if (!validation.success) {
        return validation;
      }

      // Validate YAML configuration
      const yamlValidation = await this.validateYamlConfig(strategyConfig.yamlConfig);
      if (!yamlValidation.success) {
        return yamlValidation;
      }

      const strategyId = `strategy_${userId}_${Date.now()}`;
      const strategy: TechnicalStrategy = {
        id: strategyId,
        userId,
        name: strategyConfig.name,
        description: strategyConfig.description,
        yamlConfig: strategyConfig.yamlConfig,
        symbols: strategyConfig.symbols,
        timeframes: strategyConfig.timeframes,
        riskLevel: strategyConfig.riskLevel,
        isActive: strategyConfig.isActive,
        createdAt: Date.now(),
        updatedAt: Date.now(),
        backtestResults: [],
        performance: {
          totalTrades: 0,
          winRate: 0,
          profitLoss: 0,
          maxDrawdown: 0,
          sharpeRatio: 0
        }
      };

      // Store strategy
      const strategyKey = `rbac:strategy:${strategyId}`;
      await this.env.ArbEdgeKV.put(strategyKey, JSON.stringify(strategy), {
        expirationTtl: 30 * 24 * 60 * 60 // 30 days
      });

      // Add to user's strategy list
      const userStrategiesKey = `rbac:user_strategies:${userId}`;
      const existingStrategies = await this.env.ArbEdgeKV.get(userStrategiesKey, 'json') || [];
      existingStrategies.push(strategyId);
      
      await this.env.ArbEdgeKV.put(userStrategiesKey, JSON.stringify(existingStrategies), {
        expirationTtl: 30 * 24 * 60 * 60
      });

      // Update strategy limits
      const limits = await this.getStrategyLimits(userId);
      if (limits) {
        limits.createdStrategies++;
        if (strategyConfig.isActive) {
          limits.activeStrategies++;
        }

        const limitsKey = `rbac:strategy_limits:${userId}`;
        await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
          expirationTtl: 86400
        });
      }

      return {
        success: true,
        message: 'Strategy created successfully',
        timestamp: Date.now(),
        data: {
          strategyId,
          strategy: {
            id: strategy.id,
            name: strategy.name,
            isActive: strategy.isActive,
            riskLevel: strategy.riskLevel
          }
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to create strategy',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get user's strategies
   */
  async getUserStrategies(
    userId: string,
    filters?: {
      isActive?: boolean;
      riskLevel?: 'low' | 'medium' | 'high';
      symbols?: string[];
    }
  ): Promise<TechnicalStrategy[]> {
    try {
      const userStrategiesKey = `rbac:user_strategies:${userId}`;
      const strategyIds = await this.env.ArbEdgeKV.get(userStrategiesKey, 'json') || [];
      
      const strategies: TechnicalStrategy[] = [];
      for (const strategyId of strategyIds) {
        const strategyKey = `rbac:strategy:${strategyId}`;
        const strategy = await this.env.ArbEdgeKV.get(strategyKey, 'json');
        if (strategy) {
          strategies.push(strategy as TechnicalStrategy);
        }
      }
      
      // Apply filters
      if (filters) {
        return this.applyStrategyFilters(strategies, filters);
      }
      
      return strategies;
    } catch (error) {
      console.error('Failed to get user strategies:', error);
      return [];
    }
  }

  /**
   * Update strategy configuration
   */
  async updateStrategy(
    userId: string,
    strategyId: string,
    updates: Partial<{
      name: string;
      description: string;
      yamlConfig: string;
      symbols: string[];
      timeframes: string[];
      riskLevel: 'low' | 'medium' | 'high';
      isActive: boolean;
    }>
  ): Promise<RBACOperationResult> {
    try {
      const strategyKey = `rbac:strategy:${strategyId}`;
      const strategy = await this.env.ArbEdgeKV.get(strategyKey, 'json') as TechnicalStrategy;
      
      if (!strategy || strategy.userId !== userId) {
        return {
          success: false,
          message: 'Strategy not found or access denied',
          timestamp: Date.now(),
          errors: ['Strategy not found or user does not have access']
        };
      }

      // Validate YAML if being updated
      if (updates.yamlConfig) {
        const yamlValidation = await this.validateYamlConfig(updates.yamlConfig);
        if (!yamlValidation.success) {
          return yamlValidation;
        }
      }

      // Check activation limits if activating
      if (updates.isActive === true && !strategy.isActive) {
        const validation = await this.validateStrategyOperation(userId, 'activate');
        if (!validation.success) {
          return validation;
        }
      }

      // Update strategy
      const updatedStrategy: TechnicalStrategy = {
        ...strategy,
        ...updates,
        updatedAt: Date.now()
      };

      await this.env.ArbEdgeKV.put(strategyKey, JSON.stringify(updatedStrategy), {
        expirationTtl: 30 * 24 * 60 * 60
      });

      // Update active strategy count if status changed
      if (updates.isActive !== undefined && updates.isActive !== strategy.isActive) {
        const limits = await this.getStrategyLimits(userId);
        if (limits) {
          if (updates.isActive) {
            limits.activeStrategies++;
          } else {
            limits.activeStrategies = Math.max(0, limits.activeStrategies - 1);
          }

          const limitsKey = `rbac:strategy_limits:${userId}`;
          await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
            expirationTtl: 86400
          });
        }
      }

      return {
        success: true,
        message: 'Strategy updated successfully',
        timestamp: Date.now(),
        data: {
          strategyId,
          updatedFields: Object.keys(updates)
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update strategy',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Delete a strategy
   */
  async deleteStrategy(userId: string, strategyId: string): Promise<RBACOperationResult> {
    try {
      const strategyKey = `rbac:strategy:${strategyId}`;
      const strategy = await this.env.ArbEdgeKV.get(strategyKey, 'json') as TechnicalStrategy;
      
      if (!strategy || strategy.userId !== userId) {
        return {
          success: false,
          message: 'Strategy not found or access denied',
          timestamp: Date.now(),
          errors: ['Strategy not found or user does not have access']
        };
      }

      // Delete strategy
      await this.env.ArbEdgeKV.delete(strategyKey);

      // Remove from user's strategy list
      const userStrategiesKey = `rbac:user_strategies:${userId}`;
      const existingStrategies = await this.env.ArbEdgeKV.get(userStrategiesKey, 'json') || [];
      const updatedStrategies = existingStrategies.filter((id: string) => id !== strategyId);
      
      await this.env.ArbEdgeKV.put(userStrategiesKey, JSON.stringify(updatedStrategies), {
        expirationTtl: 30 * 24 * 60 * 60
      });

      // Update strategy limits
      const limits = await this.getStrategyLimits(userId);
      if (limits) {
        limits.createdStrategies = Math.max(0, limits.createdStrategies - 1);
        if (strategy.isActive) {
          limits.activeStrategies = Math.max(0, limits.activeStrategies - 1);
        }

        const limitsKey = `rbac:strategy_limits:${userId}`;
        await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
          expirationTtl: 86400
        });
      }

      return {
        success: true,
        message: 'Strategy deleted successfully',
        timestamp: Date.now(),
        data: { strategyId }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to delete strategy',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Start strategy backtest
   */
  async startBacktest(
    userId: string,
    strategyId: string,
    backtestConfig: {
      startDate: number;
      endDate: number;
      initialCapital: number;
      symbols?: string[];
      timeframes?: string[];
    }
  ): Promise<RBACOperationResult> {
    try {
      // Validate operation
      const validation = await this.validateStrategyOperation(userId, 'backtest');
      if (!validation.success) {
        return validation;
      }

      const strategyKey = `rbac:strategy:${strategyId}`;
      const strategy = await this.env.ArbEdgeKV.get(strategyKey, 'json') as TechnicalStrategy;
      
      if (!strategy || strategy.userId !== userId) {
        return {
          success: false,
          message: 'Strategy not found or access denied',
          timestamp: Date.now(),
          errors: ['Strategy not found or user does not have access']
        };
      }

      const backtestId = `backtest_${strategyId}_${Date.now()}`;
      const backtest = {
        id: backtestId,
        strategyId,
        userId,
        config: backtestConfig,
        status: 'running',
        startedAt: Date.now(),
        completedAt: null,
        results: null,
        progress: 0
      };

      // Store backtest
      const backtestKey = `rbac:backtest:${backtestId}`;
      await this.env.ArbEdgeKV.put(backtestKey, JSON.stringify(backtest), {
        expirationTtl: 7 * 24 * 60 * 60 // 7 days
      });

      // Add to user's backtest list
      const userBacktestsKey = `rbac:user_backtests:${userId}`;
      const existingBacktests = await this.env.ArbEdgeKV.get(userBacktestsKey, 'json') || [];
      existingBacktests.push(backtestId);
      
      await this.env.ArbEdgeKV.put(userBacktestsKey, JSON.stringify(existingBacktests), {
        expirationTtl: 7 * 24 * 60 * 60
      });

      // Update running backtest count
      const limits = await this.getStrategyLimits(userId);
      if (limits) {
        limits.runningBacktests++;
        const limitsKey = `rbac:strategy_limits:${userId}`;
        await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
          expirationTtl: 86400
        });
      }

      // In production, this would trigger the actual backtest process
      // For now, we'll simulate it with a timeout
      this.simulateBacktest(backtestId, userId);

      return {
        success: true,
        message: 'Backtest started successfully',
        timestamp: Date.now(),
        data: {
          backtestId,
          strategyId,
          estimatedDuration: '5-10 minutes'
        }
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to start backtest',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get backtest results
   */
  async getBacktestResults(userId: string, backtestId: string): Promise<any> {
    try {
      const backtestKey = `rbac:backtest:${backtestId}`;
      const backtest = await this.env.ArbEdgeKV.get(backtestKey, 'json');
      
      if (!backtest || backtest.userId !== userId) {
        return null;
      }
      
      return backtest;
    } catch (error) {
      console.error('Failed to get backtest results:', error);
      return null;
    }
  }

  /**
   * Get user's backtests
   */
  async getUserBacktests(userId: string): Promise<any[]> {
    try {
      const userBacktestsKey = `rbac:user_backtests:${userId}`;
      const backtestIds = await this.env.ArbEdgeKV.get(userBacktestsKey, 'json') || [];
      
      const backtests = [];
      for (const backtestId of backtestIds) {
        const backtestKey = `rbac:backtest:${backtestId}`;
        const backtest = await this.env.ArbEdgeKV.get(backtestKey, 'json');
        if (backtest) {
          backtests.push(backtest);
        }
      }
      
      return backtests.sort((a, b) => b.startedAt - a.startedAt);
    } catch (error) {
      console.error('Failed to get user backtests:', error);
      return [];
    }
  }

  /**
   * Update strategy limits
   */
  async updateStrategyLimits(
    userId: string,
    newLimits: Partial<StrategyLimits>
  ): Promise<RBACOperationResult> {
    try {
      const existingLimits = await this.getStrategyLimits(userId);
      
      if (!existingLimits) {
        return {
          success: false,
          message: 'Strategy limits not found',
          timestamp: Date.now(),
          errors: ['User strategy limits not initialized']
        };
      }

      const updatedLimits: StrategyLimits = {
        ...existingLimits,
        ...newLimits
      };

      // Store updated limits
      const key = `rbac:strategy_limits:${userId}`;
      await this.env.ArbEdgeKV.put(key, JSON.stringify(updatedLimits), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Strategy limits updated successfully',
        timestamp: Date.now(),
        data: updatedLimits
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update strategy limits',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get tier-based strategy limits
   */
  private getTierLimits(tier: SubscriptionTierType): {
    maxStrategies: number;
    maxConcurrentBacktests: number;
  } {
    switch (tier) {
      case 'free':
        return { maxStrategies: 3, maxConcurrentBacktests: 1 };
      case 'pro':
        return { maxStrategies: 10, maxConcurrentBacktests: 3 };
      case 'ultra':
        return { maxStrategies: 50, maxConcurrentBacktests: 10 };
      case 'enterprise':
        return { maxStrategies: -1, maxConcurrentBacktests: -1 }; // unlimited
      default:
        return { maxStrategies: 3, maxConcurrentBacktests: 1 };
    }
  }

  /**
   * Validate YAML configuration
   */
  private async validateYamlConfig(yamlConfig: string): Promise<RBACOperationResult> {
    try {
      // Basic YAML validation - in production, use a proper YAML parser
      if (!yamlConfig || yamlConfig.trim().length === 0) {
        return {
          success: false,
          message: 'YAML configuration is required',
          timestamp: Date.now(),
          errors: ['Empty YAML configuration']
        };
      }

      // Check for required sections
      const requiredSections = ['strategy', 'indicators', 'signals', 'risk_management'];
      const missingSections = requiredSections.filter(section => 
        !yamlConfig.includes(`${section}:`)
      );

      if (missingSections.length > 0) {
        return {
          success: false,
          message: 'YAML configuration missing required sections',
          timestamp: Date.now(),
          errors: [`Missing sections: ${missingSections.join(', ')}`]
        };
      }

      // Additional validation can be added here
      // - Check for valid indicator names
      // - Validate parameter ranges
      // - Check for circular dependencies

      return {
        success: true,
        message: 'YAML configuration is valid',
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to validate YAML configuration',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Apply filters to strategies
   */
  private applyStrategyFilters(
    strategies: TechnicalStrategy[],
    filters: {
      isActive?: boolean;
      riskLevel?: 'low' | 'medium' | 'high';
      symbols?: string[];
    }
  ): TechnicalStrategy[] {
    return strategies.filter(strategy => {
      if (filters.isActive !== undefined && strategy.isActive !== filters.isActive) {
        return false;
      }
      
      if (filters.riskLevel && strategy.riskLevel !== filters.riskLevel) {
        return false;
      }
      
      if (filters.symbols && filters.symbols.length > 0) {
        const hasMatchingSymbol = strategy.symbols.some(symbol => 
          filters.symbols!.includes(symbol)
        );
        if (!hasMatchingSymbol) {
          return false;
        }
      }
      
      return true;
    });
  }

  /**
   * Simulate backtest execution (in production, this would be a real backtest engine)
   */
  private async simulateBacktest(backtestId: string, userId: string): Promise<void> {
    try {
      // Simulate backtest execution with random results
      setTimeout(async () => {
        const backtestKey = `rbac:backtest:${backtestId}`;
        const backtest = await this.env.ArbEdgeKV.get(backtestKey, 'json');
        
        if (backtest) {
          // Generate mock results
          const results = {
            totalTrades: Math.floor(Math.random() * 100) + 50,
            winRate: Math.random() * 0.4 + 0.4, // 40-80%
            profitLoss: (Math.random() - 0.3) * 10000, // -3000 to 7000
            maxDrawdown: Math.random() * 0.2, // 0-20%
            sharpeRatio: Math.random() * 2 + 0.5, // 0.5-2.5
            trades: [] // In production, this would contain detailed trade data
          };

          backtest.status = 'completed';
          backtest.completedAt = Date.now();
          backtest.results = results;
          backtest.progress = 100;

          await this.env.ArbEdgeKV.put(backtestKey, JSON.stringify(backtest), {
            expirationTtl: 7 * 24 * 60 * 60
          });

          // Update running backtest count
          const limits = await this.getStrategyLimits(userId);
          if (limits) {
            limits.runningBacktests = Math.max(0, limits.runningBacktests - 1);
            const limitsKey = `rbac:strategy_limits:${userId}`;
            await this.env.ArbEdgeKV.put(limitsKey, JSON.stringify(limits), {
              expirationTtl: 86400
            });
          }
        }
      }, 30000); // 30 seconds simulation
    } catch (error) {
      console.error('Failed to simulate backtest:', error);
    }
  }
}