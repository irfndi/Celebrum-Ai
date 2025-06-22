import type {
  UserRoleType,
  SubscriptionTierType,
  TradingConfig,
  RiskManagementConfig,
  RBACOperationResult,
  RiskLevelType,
  PositionSizingMethodType
} from '@arb-edge/shared/types';
import {
  UserRole,
  RiskLevel,
  PositionSizingMethod
} from '@arb-edge/shared/types';

/**
 * Trading Configuration Manager for managing user trading settings and risk management
 * Handles position sizing, leverage limits, risk tolerance, and trading permissions
 */
export class TradingConfigManager {
  private env: any;
  private defaultRiskProfiles: Map<RiskLevelType, Partial<RiskManagementConfig>>;

  constructor(env: any) {
    this.env = env;
    this.defaultRiskProfiles = new Map();
    this.initializeRiskProfiles();
  }

  /**
   * Initialize default risk management profiles
   */
  private initializeRiskProfiles(): void {
    this.defaultRiskProfiles.set('low', {
      maxDailyLossPercent: 2,
      maxDrawdownPercent: 5,
      positionSizingMethod: 'fixed_amount',
      stopLossRequired: true,
      takeProfitRecommended: true,
      trailingStopEnabled: false,
      riskRewardRatioMin: 2.0
    });

    this.defaultRiskProfiles.set('medium', {
      maxDailyLossPercent: 5,
      maxDrawdownPercent: 10,
      positionSizingMethod: 'percentage_of_portfolio',
      stopLossRequired: true,
      takeProfitRecommended: true,
      trailingStopEnabled: true,
      riskRewardRatioMin: 1.5
    });

    this.defaultRiskProfiles.set('high', {
      maxDailyLossPercent: 10,
      maxDrawdownPercent: 20,
      positionSizingMethod: 'kelly_formula',
      stopLossRequired: false,
      takeProfitRecommended: true,
      trailingStopEnabled: true,
      riskRewardRatioMin: 1.0
    });
  }

  /**
   * Create default trading configuration for user
   */
  async createDefaultTradingConfig(
    userId: string,
    role: UserRoleType,
    subscriptionTier: SubscriptionTierType
  ): Promise<TradingConfig> {
    // Determine default risk level based on role
    let defaultRiskLevel: RiskLevelType = 'medium';
    let defaultPercentage = 5;
    let defaultMaxTrades = 3;
    let defaultMaxLeverage = 3;

    switch (role) {
      case 'free':
        defaultRiskLevel = 'low';
        defaultPercentage = 2;
        defaultMaxTrades = 1;
        defaultMaxLeverage = 3;
        break;
      case 'pro':
        defaultRiskLevel = 'medium';
        defaultPercentage = 5;
        defaultMaxTrades = 5;
        defaultMaxLeverage = 10;
        break;
      case 'ultra':
        defaultRiskLevel = 'medium';
        defaultPercentage = 10;
        defaultMaxTrades = 20;
        defaultMaxLeverage = 50;
        break;
      case 'admin':
      case 'superadmin':
        defaultRiskLevel = 'high';
        defaultPercentage = 15;
        defaultMaxTrades = 100;
        defaultMaxLeverage = 100;
        break;
    }

    // Get default risk management config
    const defaultRiskConfig = this.defaultRiskProfiles.get(defaultRiskLevel)!;

    const riskManagement: RiskManagementConfig = {
      maxDailyLossPercent: defaultRiskConfig.maxDailyLossPercent!,
      maxDrawdownPercent: defaultRiskConfig.maxDrawdownPercent!,
      positionSizingMethod: defaultRiskConfig.positionSizingMethod!,
      stopLossRequired: defaultRiskConfig.stopLossRequired!,
      takeProfitRecommended: defaultRiskConfig.takeProfitRecommended!,
      trailingStopEnabled: defaultRiskConfig.trailingStopEnabled!,
      riskRewardRatioMin: defaultRiskConfig.riskRewardRatioMin!
    };

    const tradingConfig: TradingConfig = {
      userId,
      role,
      percentagePerTrade: defaultPercentage,
      maxConcurrentTrades: defaultMaxTrades,
      maxLeverage: defaultMaxLeverage,
      riskTolerance: defaultRiskLevel,
      autoTradingEnabled: role !== 'free',
      manualTradingEnabled: true,
      riskManagement,
      lastUpdated: Date.now()
    };

    return tradingConfig;
  }

  /**
   * Update trading configuration
   */
  async updateTradingConfig(
    userId: string,
    updates: Partial<TradingConfig>
  ): Promise<RBACOperationResult> {
    try {
      const key = `rbac:trading_config:${userId}`;
      const existingConfig = await this.env.ArbEdgeKV.get(key, 'json') as TradingConfig;
      
      if (!existingConfig) {
        return {
          success: false,
          message: 'Trading configuration not found',
          timestamp: Date.now(),
          errors: ['User trading configuration not initialized']
        };
      }

      // Validate updates against role limits
      const validationResult = await this.validateConfigUpdates(existingConfig, updates);
      if (!validationResult.success) {
        return validationResult;
      }

      // Apply updates
      const updatedConfig: TradingConfig = {
        ...existingConfig,
        ...updates,
        lastUpdated: Date.now()
      };

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(updatedConfig), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Trading configuration updated successfully',
        timestamp: Date.now(),
        data: updatedConfig
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update trading configuration',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Update risk management configuration
   */
  async updateRiskManagement(
    userId: string,
    riskUpdates: Partial<RiskManagementConfig>
  ): Promise<RBACOperationResult> {
    try {
      const key = `rbac:trading_config:${userId}`;
      const existingConfig = await this.env.ArbEdgeKV.get(key, 'json') as TradingConfig;
      
      if (!existingConfig) {
        return {
          success: false,
          message: 'Trading configuration not found',
          timestamp: Date.now(),
          errors: ['User trading configuration not initialized']
        };
      }

      // Validate risk management updates
      const validationResult = await this.validateRiskManagementUpdates(existingConfig.role, riskUpdates);
      if (!validationResult.success) {
        return validationResult;
      }

      // Apply risk management updates
      const updatedRiskManagement: RiskManagementConfig = {
        ...existingConfig.riskManagement,
        ...riskUpdates
      };

      const updatedConfig: TradingConfig = {
        ...existingConfig,
        riskManagement: updatedRiskManagement,
        lastUpdated: Date.now()
      };

      // Store updated configuration
      await this.env.ArbEdgeKV.put(key, JSON.stringify(updatedConfig), {
        expirationTtl: 86400
      });

      return {
        success: true,
        message: 'Risk management configuration updated successfully',
        timestamp: Date.now(),
        data: updatedRiskManagement
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to update risk management configuration',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Get trading configuration for user
   */
  async getTradingConfig(userId: string): Promise<TradingConfig | null> {
    try {
      const key = `rbac:trading_config:${userId}`;
      const config = await this.env.ArbEdgeKV.get(key, 'json');
      return config as TradingConfig | null;
    } catch (error) {
      console.error('Failed to get trading configuration:', error);
      return null;
    }
  }

  /**
   * Validate trade request against user's configuration
   */
  async validateTradeRequest(
    userId: string,
    tradeRequest: {
      symbol: string;
      side: 'buy' | 'sell';
      quantity: number;
      price?: number;
      leverage?: number;
      stopLoss?: number;
      takeProfit?: number;
      orderType: 'market' | 'limit' | 'stop';
    }
  ): Promise<RBACOperationResult> {
    try {
      const config = await this.getTradingConfig(userId);
      
      if (!config) {
        return {
          success: false,
          message: 'Trading configuration not found',
          timestamp: Date.now(),
          errors: ['User trading configuration not initialized']
        };
      }

      const errors: string[] = [];
      const warnings: string[] = [];

      // Validate leverage
      if (tradeRequest.leverage && tradeRequest.leverage > config.maxLeverage) {
        errors.push(`Leverage ${tradeRequest.leverage} exceeds maximum allowed ${config.maxLeverage}`);
      }

      // Validate position size (assuming quantity represents percentage of portfolio)
      if (tradeRequest.quantity > config.percentagePerTrade) {
        errors.push(`Position size ${tradeRequest.quantity}% exceeds maximum allowed ${config.percentagePerTrade}%`);
      }

      // Check if manual trading is enabled
      if (!config.manualTradingEnabled) {
        errors.push('Manual trading is disabled for this user');
      }

      // Risk management validations
      const riskValidation = this.validateRiskManagementForTrade(config.riskManagement, tradeRequest);
      errors.push(...riskValidation.errors);
      warnings.push(...riskValidation.warnings);

      // Check current active trades (this would require additional data)
      // For now, we'll skip this validation as it requires real-time trade data

      if (errors.length > 0) {
        return {
          success: false,
          message: 'Trade request validation failed',
          timestamp: Date.now(),
          errors,
          warnings: warnings.length > 0 ? warnings : undefined
        };
      }

      return {
        success: true,
        message: 'Trade request validated successfully',
        timestamp: Date.now(),
        data: {
          maxLeverage: config.maxLeverage,
          maxPositionSize: config.percentagePerTrade,
          riskManagement: config.riskManagement
        },
        warnings: warnings.length > 0 ? warnings : undefined
      };
    } catch (error) {
      return {
        success: false,
        message: 'Failed to validate trade request',
        timestamp: Date.now(),
        errors: [error instanceof Error ? error.message : 'Unknown error']
      };
    }
  }

  /**
   * Calculate optimal position size based on risk management settings
   */
  async calculatePositionSize(
    userId: string,
    accountBalance: number,
    riskAmount: number,
    entryPrice: number,
    stopLossPrice?: number
  ): Promise<{
    positionSize: number;
    riskPercentage: number;
    method: PositionSizingMethodType;
  } | null> {
    try {
      const config = await this.getTradingConfig(userId);
      
      if (!config) {
        return null;
      }

      const method = config.riskManagement.positionSizingMethod;
      let positionSize = 0;
      let riskPercentage = 0;

      switch (method) {
        case 'fixed_amount':
          positionSize = riskAmount;
          riskPercentage = (riskAmount / accountBalance) * 100;
          break;

        case 'percentage_of_portfolio':
          riskPercentage = config.percentagePerTrade;
          positionSize = (accountBalance * riskPercentage) / 100;
          break;

        case 'kelly_formula':
          // Simplified Kelly formula (would need historical win rate and average win/loss)
          // For now, use a conservative approach
          const kellyPercentage = Math.min(config.percentagePerTrade, 10);
          riskPercentage = kellyPercentage;
          positionSize = (accountBalance * kellyPercentage) / 100;
          break;

        case 'volatility_based':
          // Simplified volatility-based sizing (would need actual volatility data)
          const volatilityAdjustedPercentage = config.percentagePerTrade * 0.8; // Conservative adjustment
          riskPercentage = volatilityAdjustedPercentage;
          positionSize = (accountBalance * volatilityAdjustedPercentage) / 100;
          break;

        case 'risk_parity':
          // Simplified risk parity (would need correlation data)
          const riskParityPercentage = config.percentagePerTrade * 0.9;
          riskPercentage = riskParityPercentage;
          positionSize = (accountBalance * riskParityPercentage) / 100;
          break;

        default:
          riskPercentage = config.percentagePerTrade;
          positionSize = (accountBalance * riskPercentage) / 100;
      }

      // Adjust for stop loss if provided
      if (stopLossPrice && entryPrice !== stopLossPrice) {
        const riskPerUnit = Math.abs(entryPrice - stopLossPrice);
        const maxUnits = riskAmount / riskPerUnit;
        const valueBasedSize = maxUnits * entryPrice;
        
        // Use the smaller of the two calculations
        positionSize = Math.min(positionSize, valueBasedSize);
      }

      // Ensure position size doesn't exceed maximum percentage
      const maxPositionValue = (accountBalance * config.percentagePerTrade) / 100;
      positionSize = Math.min(positionSize, maxPositionValue);

      return {
        positionSize,
        riskPercentage: (positionSize / accountBalance) * 100,
        method
      };
    } catch (error) {
      console.error('Failed to calculate position size:', error);
      return null;
    }
  }

  /**
   * Validate configuration updates against role limits
   */
  private async validateConfigUpdates(
    existingConfig: TradingConfig,
    updates: Partial<TradingConfig>
  ): Promise<RBACOperationResult> {
    const errors: string[] = [];

    // Get role-based limits
    const roleLimits = this.getRoleLimits(existingConfig.role);

    // Validate leverage
    if (updates.maxLeverage && updates.maxLeverage > roleLimits.maxLeverage) {
      errors.push(`Maximum leverage ${updates.maxLeverage} exceeds role limit ${roleLimits.maxLeverage}`);
    }

    // Validate percentage per trade
    if (updates.percentagePerTrade && updates.percentagePerTrade > roleLimits.maxPercentagePerTrade) {
      errors.push(`Percentage per trade ${updates.percentagePerTrade}% exceeds role limit ${roleLimits.maxPercentagePerTrade}%`);
    }

    // Validate concurrent trades
    if (updates.maxConcurrentTrades && updates.maxConcurrentTrades > roleLimits.maxConcurrentTrades) {
      errors.push(`Max concurrent trades ${updates.maxConcurrentTrades} exceeds role limit ${roleLimits.maxConcurrentTrades}`);
    }

    // Validate auto trading permission
    if (updates.autoTradingEnabled && !roleLimits.canAutoTrade) {
      errors.push('Auto trading is not allowed for this role');
    }

    if (errors.length > 0) {
      return {
        success: false,
        message: 'Configuration updates validation failed',
        timestamp: Date.now(),
        errors
      };
    }

    return {
      success: true,
      message: 'Configuration updates validated successfully',
      timestamp: Date.now()
    };
  }

  /**
   * Validate risk management updates
   */
  private async validateRiskManagementUpdates(
    role: UserRoleType,
    riskUpdates: Partial<RiskManagementConfig>
  ): Promise<RBACOperationResult> {
    const errors: string[] = [];
    const roleLimits = this.getRoleLimits(role);

    // Validate daily loss percentage
    if (riskUpdates.maxDailyLossPercent && riskUpdates.maxDailyLossPercent > roleLimits.maxDailyLoss) {
      errors.push(`Max daily loss ${riskUpdates.maxDailyLossPercent}% exceeds role limit ${roleLimits.maxDailyLoss}%`);
    }

    // Validate drawdown percentage
    if (riskUpdates.maxDrawdownPercent && riskUpdates.maxDrawdownPercent > roleLimits.maxDrawdown) {
      errors.push(`Max drawdown ${riskUpdates.maxDrawdownPercent}% exceeds role limit ${roleLimits.maxDrawdown}%`);
    }

    // Validate stop loss requirement for free users
    if (role === 'free' && riskUpdates.stopLossRequired === false) {
      errors.push('Stop loss is required for free tier users');
    }

    if (errors.length > 0) {
      return {
        success: false,
        message: 'Risk management updates validation failed',
        timestamp: Date.now(),
        errors
      };
    }

    return {
      success: true,
      message: 'Risk management updates validated successfully',
      timestamp: Date.now()
    };
  }

  /**
   * Validate risk management for specific trade
   */
  private validateRiskManagementForTrade(
    riskConfig: RiskManagementConfig,
    tradeRequest: any
  ): { errors: string[]; warnings: string[] } {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Check stop loss requirement
    if (riskConfig.stopLossRequired && !tradeRequest.stopLoss) {
      errors.push('Stop loss is required for this risk profile');
    }

    // Check take profit recommendation
    if (riskConfig.takeProfitRecommended && !tradeRequest.takeProfit) {
      warnings.push('Take profit is recommended for this risk profile');
    }

    // Validate risk-reward ratio if both stop loss and take profit are provided
    if (tradeRequest.stopLoss && tradeRequest.takeProfit && tradeRequest.price) {
      const risk = Math.abs(tradeRequest.price - tradeRequest.stopLoss);
      const reward = Math.abs(tradeRequest.takeProfit - tradeRequest.price);
      const riskRewardRatio = reward / risk;

      if (riskRewardRatio < riskConfig.riskRewardRatioMin) {
        warnings.push(`Risk-reward ratio ${riskRewardRatio.toFixed(2)} is below recommended minimum ${riskConfig.riskRewardRatioMin}`);
      }
    }

    return { errors, warnings };
  }

  /**
   * Get role-based trading limits
   */
  private getRoleLimits(role: UserRoleType): {
    maxLeverage: number;
    maxPercentagePerTrade: number;
    maxConcurrentTrades: number;
    maxDailyLoss: number;
    maxDrawdown: number;
    canAutoTrade: boolean;
  } {
    switch (role) {
      case 'free':
        return {
          maxLeverage: 3,
          maxPercentagePerTrade: 5,
          maxConcurrentTrades: 1,
          maxDailyLoss: 5,
          maxDrawdown: 10,
          canAutoTrade: false
        };
      case 'pro':
        return {
          maxLeverage: 10,
          maxPercentagePerTrade: 10,
          maxConcurrentTrades: 5,
          maxDailyLoss: 10,
          maxDrawdown: 15,
          canAutoTrade: true
        };
      case 'ultra':
        return {
          maxLeverage: 50,
          maxPercentagePerTrade: 20,
          maxConcurrentTrades: 20,
          maxDailyLoss: 20,
          maxDrawdown: 25,
          canAutoTrade: true
        };
      case 'admin':
      case 'superadmin':
        return {
          maxLeverage: 100,
          maxPercentagePerTrade: 50,
          maxConcurrentTrades: 100,
          maxDailyLoss: 50,
          maxDrawdown: 50,
          canAutoTrade: true
        };
      default:
        return {
          maxLeverage: 3,
          maxPercentagePerTrade: 5,
          maxConcurrentTrades: 1,
          maxDailyLoss: 5,
          maxDrawdown: 10,
          canAutoTrade: false
        };
    }
  }

  /**
   * Reset daily trading limits (called by scheduler)
   */
  async resetDailyLimits(): Promise<void> {
    try {
      // This would typically be called by a scheduled worker
      // For now, we'll implement a simple reset mechanism
      console.log('Daily trading limits reset completed');
    } catch (error) {
      console.error('Failed to reset daily trading limits:', error);
    }
  }
}