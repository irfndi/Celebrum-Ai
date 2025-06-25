import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { RBACService } from '../services/rbac';
import { ApiAccessManager } from '../services/api-access-manager';
import { TradingConfigManager } from '../services/trading-config-manager';
import { ArbitrageOpportunityManager } from '../services/arbitrage-opportunity-manager';
import { TechnicalStrategyManager } from '../services/technical-strategy-manager';
import { FeatureFlagManager } from '../services/feature-flag-manager';
import type { Env } from '@celebrum-ai/shared';
import {
  Permission
} from '@celebrum-ai/shared/types';
import type {
  UserRoleType,
  SubscriptionTierType,
  ExchangeIdType,
  PermissionType,
  RiskLevelType,
  PositionSizingMethodType
} from '@celebrum-ai/shared/types';

type Variables = {
  rbacService: RBACService;
  apiAccessManager: ApiAccessManager;
  tradingConfigManager: TradingConfigManager;
  arbitrageOpportunityManager: ArbitrageOpportunityManager;
  technicalStrategyManager: TechnicalStrategyManager;
  featureFlagManager: FeatureFlagManager;
  currentUser: {
    userId: string;
    role: UserRoleType;
    subscriptionTier: SubscriptionTierType;
  };
};

const rbacRoutes = new Hono<{ Bindings: Env; Variables: Variables }>();

// CORS middleware
rbacRoutes.use('*', cors({
  origin: ['http://localhost:3000', 'https://celebrum-ai.com'],
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization'],
  credentials: true
}));

// Middleware to initialize services
rbacRoutes.use('*', async (c, next) => {
  c.set('rbacService', new RBACService(c.env));
  c.set('apiAccessManager', new ApiAccessManager(c.env));
  c.set('tradingConfigManager', new TradingConfigManager(c.env));
  c.set('arbitrageOpportunityManager', new ArbitrageOpportunityManager(c.env));
  c.set('technicalStrategyManager', new TechnicalStrategyManager(c.env));
  c.set('featureFlagManager', new FeatureFlagManager(c.env));
  await next();
});

// Authentication middleware
rbacRoutes.use('/api/*', async (c, next) => {
  const authHeader = c.req.header('Authorization');
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return c.json({ error: 'Missing or invalid authorization header' }, 401);
  }

  const token = authHeader.substring(7);
  try {
    // In production, verify JWT token here
    // For now, we'll extract user info from a simple format
    const userInfo = JSON.parse(atob(token));
    c.set('currentUser', userInfo);
    await next();
  } catch (error) {
    return c.json({ error: 'Invalid token' }, 401);
  }
});

// User Management Routes

/**
 * Register a new user
 */
rbacRoutes.post('/api/users/register', async (c) => {
  try {
    const { userId, role, subscriptionTier } = await c.req.json();
    const rbacService = c.get('rbacService') as RBACService;
    
    const result = await rbacService.registerUser(userId, role, subscriptionTier);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'User registered successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to register user',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get user access summary
 */
rbacRoutes.get('/api/users/:userId/access-summary', async (c) => {
  try {
    const userId = c.req.param('userId');
    const rbacService = c.get('rbacService') as RBACService;
    
    const summary = await rbacService.getUserAccessSummary(userId);
    
    if (summary) {
      return c.json({
        success: true,
        data: summary
      });
    } else {
      return c.json({
        success: false,
        message: 'User access summary not found'
      }, 404);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get user access summary',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Update user role
 */
rbacRoutes.put('/api/users/:userId/role', async (c) => {
  try {
    const userId = c.req.param('userId');
    const { newRole, newTier } = await c.req.json();
    const currentUser = c.get('currentUser');
    const rbacService = c.get('rbacService') as RBACService;
    
    // Check if current user has permission to update roles
    const currentUserSummary = await rbacService.getUserAccessSummary(currentUser.userId);
    if (!currentUserSummary) {
      return c.json({
        success: false,
        message: 'Current user not found'
      }, 404);
    }
    
    const hasPermission = rbacService.hasPermission(
      currentUserSummary.role,
      Permission.ADMIN_USER_MANAGEMENT
    );
    
    if (!hasPermission) {
      return c.json({
        success: false,
        message: 'Insufficient permissions to update user roles'
      }, 403);
    }
    
    const result = await rbacService.updateUserRole(userId, newRole, newTier || 'free');
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'User role updated successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to update user role',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// API Access Management Routes

/**
 * Get user's API configurations
 */
rbacRoutes.get('/api/users/:userId/api-access', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own API configs unless they're admin
    if (userId !== currentUser.userId && !['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const apiAccessManager = c.get('apiAccessManager') as ApiAccessManager;
    const apiAccess = await apiAccessManager.getApiAccessSummary(userId);
    
    return c.json({
      success: true,
      data: apiAccess
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get API access',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Add API configuration
 */
rbacRoutes.post('/api/users/:userId/api-access', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    const { type, name, credentials } = await c.req.json();
    
    // Users can only manage their own API configs
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const apiAccessManager = c.get('apiAccessManager') as ApiAccessManager;
    let result;
    
    if (type === 'exchange') {
      result = await apiAccessManager.addExchangeApi(
        userId,
        name,
        credentials.apiKey,
        credentials.apiSecret,
        credentials.passphrase,
        credentials.sandbox || false
      );
    } else if (type === 'ai') {
      result = await apiAccessManager.addAiApi(
        userId,
        name,
        credentials.apiKey,
        credentials.model,
        credentials.endpoint
      );
    } else {
      return c.json({
        success: false,
        message: 'Invalid API type'
      }, 400);
    }
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'API configuration added successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to add API configuration',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Remove API configuration
 */
rbacRoutes.delete('/api/users/:userId/api-access/:apiId', async (c) => {
  try {
    const userId = c.req.param('userId');
    const apiId = c.req.param('apiId');
    const currentUser = c.get('currentUser');
    
    // Users can only manage their own API configs
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const apiAccessManager = c.get('apiAccessManager') as ApiAccessManager;
    const { type } = c.req.query();
    let result;
    
    if (type === 'exchange') {
      result = await apiAccessManager.removeExchangeApi(userId, apiId as ExchangeIdType);
    } else if (type === 'ai') {
      result = await apiAccessManager.removeAiApi(userId, apiId);
    } else {
      return c.json({
        success: false,
        message: 'Invalid API type or type parameter missing'
      }, 400);
    }
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'API configuration removed successfully'
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to remove API configuration',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Trading Configuration Routes

/**
 * Get user's trading configuration
 */
rbacRoutes.get('/api/users/:userId/trading-config', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own trading config
    if (userId !== currentUser.userId && !['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const tradingConfigManager = c.get('tradingConfigManager') as TradingConfigManager;
    const config = await tradingConfigManager.getTradingConfig(userId);
    
    return c.json({
      success: true,
      data: config
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get trading configuration',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Update trading configuration
 */
rbacRoutes.put('/api/users/:userId/trading-config', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    const updates = await c.req.json();
    
    // Users can only update their own trading config
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const tradingConfigManager = c.get('tradingConfigManager') as TradingConfigManager;
    const result = await tradingConfigManager.updateTradingConfig(userId, updates);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Trading configuration updated successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to update trading configuration',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Validate trade request
 */
rbacRoutes.post('/api/users/:userId/trading-config/validate-trade', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    const tradeRequest = await c.req.json();
    
    // Users can only validate their own trades
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const tradingConfigManager = c.get('tradingConfigManager') as TradingConfigManager;
    const result = await tradingConfigManager.validateTradeRequest(userId, tradeRequest);
    
    return c.json({
      success: true,
      data: result
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to validate trade request',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Arbitrage Opportunity Routes

/**
 * Get available opportunities for user
 */
rbacRoutes.get('/api/users/:userId/opportunities', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own opportunities
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const filters = {
      minProfitPercent: c.req.query('minProfitPercent') ? parseFloat(c.req.query('minProfitPercent')!) : undefined,
      maxRisk: c.req.query('maxRisk') as 'low' | 'medium' | 'high' | undefined,
      exchanges: c.req.query('exchanges')?.split(','),
      symbols: c.req.query('symbols')?.split(',')
    };
    
    const arbitrageOpportunityManager = c.get('arbitrageOpportunityManager') as ArbitrageOpportunityManager;
    const opportunities = await arbitrageOpportunityManager.getAvailableOpportunities(
      userId,
      currentUser.role,
      filters
    );
    
    return c.json({
      success: true,
      data: opportunities
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get opportunities',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Record opportunity execution
 */
rbacRoutes.post('/api/users/:userId/opportunities/:opportunityId/execute', async (c) => {
  try {
    const userId = c.req.param('userId');
    const opportunityId = c.req.param('opportunityId');
    const currentUser = c.get('currentUser');
    const { success, profitLoss, executionTime } = await c.req.json();
    
    // Users can only execute their own opportunities
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const arbitrageOpportunityManager = c.get('arbitrageOpportunityManager') as ArbitrageOpportunityManager;
    const result = await arbitrageOpportunityManager.recordOpportunityExecution(
      userId,
      opportunityId,
      success,
      profitLoss,
      executionTime
    );
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Opportunity execution recorded successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to record opportunity execution',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Create opportunity alert
 */
rbacRoutes.post('/api/users/:userId/opportunity-alerts', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    const alertConfig = await c.req.json();
    
    // Users can only create their own alerts
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const arbitrageOpportunityManager = c.get('arbitrageOpportunityManager') as ArbitrageOpportunityManager;
    const result = await arbitrageOpportunityManager.createOpportunityAlert(userId, alertConfig);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Opportunity alert created successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to create opportunity alert',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get user's opportunity alerts
 */
rbacRoutes.get('/api/users/:userId/opportunity-alerts', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own alerts
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const arbitrageOpportunityManager = c.get('arbitrageOpportunityManager') as ArbitrageOpportunityManager;
    const alerts = await arbitrageOpportunityManager.getUserOpportunityAlerts(userId);
    
    return c.json({
      success: true,
      data: alerts
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get opportunity alerts',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Technical Strategy Routes

/**
 * Get user's strategies
 */
rbacRoutes.get('/api/users/:userId/strategies', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own strategies
    if (userId !== currentUser.userId && !['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const filters = {
      isActive: c.req.query('isActive') ? c.req.query('isActive') === 'true' : undefined,
      riskLevel: c.req.query('riskLevel') as 'low' | 'medium' | 'high' | undefined,
      symbols: c.req.query('symbols')?.split(',')
    };
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const strategies = await technicalStrategyManager.getUserStrategies(userId, filters);
    
    return c.json({
      success: true,
      data: strategies
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get strategies',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Create new strategy
 */
rbacRoutes.post('/api/users/:userId/strategies', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    const strategyConfig = await c.req.json();
    
    // Users can only create their own strategies
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const result = await technicalStrategyManager.createStrategy(userId, strategyConfig);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Strategy created successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to create strategy',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Update strategy
 */
rbacRoutes.put('/api/users/:userId/strategies/:strategyId', async (c) => {
  try {
    const userId = c.req.param('userId');
    const strategyId = c.req.param('strategyId');
    const currentUser = c.get('currentUser');
    const updates = await c.req.json();
    
    // Users can only update their own strategies
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const result = await technicalStrategyManager.updateStrategy(userId, strategyId, updates);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Strategy updated successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to update strategy',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Delete strategy
 */
rbacRoutes.delete('/api/users/:userId/strategies/:strategyId', async (c) => {
  try {
    const userId = c.req.param('userId');
    const strategyId = c.req.param('strategyId');
    const currentUser = c.get('currentUser');
    
    // Users can only delete their own strategies
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const result = await technicalStrategyManager.deleteStrategy(userId, strategyId);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Strategy deleted successfully'
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to delete strategy',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Start strategy backtest
 */
rbacRoutes.post('/api/users/:userId/strategies/:strategyId/backtest', async (c) => {
  try {
    const userId = c.req.param('userId');
    const strategyId = c.req.param('strategyId');
    const currentUser = c.get('currentUser');
    const backtestConfig = await c.req.json();
    
    // Users can only backtest their own strategies
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const result = await technicalStrategyManager.startBacktest(userId, strategyId, backtestConfig);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Backtest started successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to start backtest',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get backtest results
 */
rbacRoutes.get('/api/users/:userId/backtests/:backtestId', async (c) => {
  try {
    const userId = c.req.param('userId');
    const backtestId = c.req.param('backtestId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own backtest results
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const results = await technicalStrategyManager.getBacktestResults(userId, backtestId);
    
    if (results) {
      return c.json({
        success: true,
        data: results
      });
    } else {
      return c.json({
        success: false,
        message: 'Backtest results not found'
      }, 404);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get backtest results',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get user's backtests
 */
rbacRoutes.get('/api/users/:userId/backtests', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own backtests
    if (userId !== currentUser.userId) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const technicalStrategyManager = c.get('technicalStrategyManager') as TechnicalStrategyManager;
    const backtests = await technicalStrategyManager.getUserBacktests(userId);
    
    return c.json({
      success: true,
      data: backtests
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get backtests',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Feature Flag Routes

/**
 * Get user's feature flags
 */
rbacRoutes.get('/api/users/:userId/feature-flags', async (c) => {
  try {
    const userId = c.req.param('userId');
    const currentUser = c.get('currentUser');
    
    // Users can only access their own feature flags
    if (userId !== currentUser.userId && !['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const flags = await featureFlagManager.getUserFeatureFlags(
      userId,
      currentUser.role,
      currentUser.subscriptionTier
    );
    
    return c.json({
      success: true,
      data: flags
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get feature flags',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Check specific feature flag
 */
rbacRoutes.get('/api/users/:userId/feature-flags/:featureKey', async (c) => {
  try {
    const userId = c.req.param('userId');
    const featureKey = c.req.param('featureKey');
    const currentUser = c.get('currentUser');
    
    // Users can only check their own feature flags
    if (userId !== currentUser.userId && !['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Access denied'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const enabled = await featureFlagManager.isFeatureEnabled(
      featureKey,
      userId,
      currentUser.role,
      currentUser.subscriptionTier
    );
    
    // Record usage for analytics
    await featureFlagManager.recordFlagUsage(featureKey, userId, enabled);
    
    return c.json({
      success: true,
      data: {
        featureKey,
        enabled
      }
    });
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to check feature flag',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Admin Routes (require admin/superadmin role)

/**
 * Set global feature flag (admin only)
 */
rbacRoutes.put('/api/admin/feature-flags/:featureKey', async (c) => {
  try {
    const featureKey = c.req.param('featureKey');
    const { enabled } = await c.req.json();
    const currentUser = c.get('currentUser');
    
    // Only admins can set global feature flags
    if (!['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Admin access required'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const result = await featureFlagManager.setGlobalFlag(featureKey, enabled, currentUser.userId);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'Global feature flag updated successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to set global feature flag',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Set user-specific feature flag override (admin only)
 */
rbacRoutes.put('/api/admin/users/:userId/feature-flags/:featureKey', async (c) => {
  try {
    const userId = c.req.param('userId');
    const featureKey = c.req.param('featureKey');
    const { enabled } = await c.req.json();
    const currentUser = c.get('currentUser');
    
    // Only admins can set user-specific feature flags
    if (!['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Admin access required'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const result = await featureFlagManager.setUserFlag(userId, featureKey, enabled, currentUser.userId);
    
    if (result.success) {
      return c.json({
        success: true,
        message: 'User feature flag updated successfully',
        data: result.data
      });
    } else {
      return c.json({
        success: false,
        message: result.message,
        errors: result.errors
      }, 400);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to set user feature flag',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get feature flag configuration (admin only)
 */
rbacRoutes.get('/api/admin/feature-flags/:featureKey/config', async (c) => {
  try {
    const featureKey = c.req.param('featureKey');
    const currentUser = c.get('currentUser');
    
    // Only admins can view feature flag configs
    if (!['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Admin access required'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const config = await featureFlagManager.getFeatureFlagConfig(featureKey);
    
    if (config) {
      return c.json({
        success: true,
        data: config
      });
    } else {
      return c.json({
        success: false,
        message: 'Feature flag configuration not found'
      }, 404);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get feature flag configuration',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

/**
 * Get feature flag usage statistics (admin only)
 */
rbacRoutes.get('/api/admin/feature-flags/:featureKey/stats', async (c) => {
  try {
    const featureKey = c.req.param('featureKey');
    const currentUser = c.get('currentUser');
    
    // Only admins can view feature flag stats
    if (!['admin', 'superadmin'].includes(currentUser.role)) {
      return c.json({
        success: false,
        message: 'Admin access required'
      }, 403);
    }
    
    const featureFlagManager = c.get('featureFlagManager') as FeatureFlagManager;
    const stats = await featureFlagManager.getFeatureFlagStats(featureKey);
    
    if (stats) {
      return c.json({
        success: true,
        data: stats
      });
    } else {
      return c.json({
        success: false,
        message: 'Feature flag statistics not found'
      }, 404);
    }
  } catch (error) {
    return c.json({
      success: false,
      message: 'Failed to get feature flag statistics',
      error: error instanceof Error ? error.message : 'Unknown error'
    }, 500);
  }
});

// Health check endpoint
rbacRoutes.get('/health', async (c) => {
  return c.json({
    success: true,
    message: 'RBAC service is healthy',
    timestamp: Date.now(),
    version: '1.0.0'
  });
});

export default rbacRoutes;