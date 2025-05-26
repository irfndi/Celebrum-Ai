# Database Migrations for ArbEdge

## Overview
This directory contains database migrations for the ArbEdge trading platform, managing schema changes and data seeding for the D1 database.

## Migration History

### Migration 001: Initial Schema (2025-01-27)
**File**: `migrations/001_initial_schema.sql`
**Status**: ✅ Applied to Production  
**Description**: Creates all initial tables for RBAC, user management, trading, and AI features

**Key Tables Created**:
- `user_profiles` - Core user data with subscription tiers (CRITICAL FOR RBAC)
- `user_trading_preferences` - Feature access control flags  
- `telegram_group_registrations` - Group-level permissions
- `audit_log` - Security event tracking (CRITICAL FOR RBAC)
- `opportunities` - Trading opportunities data
- `positions` - User trading positions
- `notifications` & `notification_templates` - Notification system
- `ai_*` tables - AI intelligence features
- `schema_migrations` - Migration tracking

### Migration 002: Indexes and Initial Data (2025-01-27)
**File**: `migrations/002_indexes_and_data.sql`  
**Status**: ✅ Applied to Production
**Description**: Adds performance indexes, views, and inserts default system data

**Features Added**:
- Performance indexes for all RBAC tables
- Views for common RBAC queries (`active_users`, `recent_opportunities`)
- Default system configuration data
- Sample notification templates for RBAC

### Migration 003: Add Superadmin User (2025-01-27)
**File**: `migrations/003_add_superadmin.sql`
**Status**: ✅ Applied to Production
**Description**: Creates superadmin user with full permissions from environment variables

**Features Added**:
- Superadmin user profile with full system access
- Complete trading preferences with all features enabled
- Comprehensive opportunity preferences with admin capabilities
- Audit log entry for superadmin creation

### Migration 004: Invitation System Foundation (2025-01-28)
**File**: `migrations/004_test_minimal.sql` *(Note: Consider renaming to `004_invitation_foundation.sql` for clarity)*
**Status**: ✅ Applied to Production
**Description**: Creates basic invitation_codes table for invitation system foundation

**Features Added**:
- `invitation_codes` table for super admin generated codes
- Basic structure for invitation-based user registration

### Migration 005: Complete Invitation System (2025-01-28)
**File**: `migrations/005_invitation_tables_only.sql`
**Status**: ✅ Applied to Production
**Description**: Completes invitation system with referral and affiliation program tables

**Features Added**:
- `invitation_usage` table for beta user tracking with 180-day expiration
- `user_referral_codes` table for personal referral codes (CRU functionality)
- `referral_usage` table for referral tracking and bonus calculation
- `affiliation_applications` table for influencer program applications
- `affiliation_programs` table for approved programs with tier management
- System configuration for invitation/referral/affiliation parameters
- 14 configuration entries for invitation system management

## Migration Status (Production)
- **Total Queries Executed**: 150+ (47 + 93 + 2 + 8)
- **Database Size**: 0.66 MB
- **Tables Created**: 28+ tables plus indexes and views
- **RBAC System**: ✅ **FULLY OPERATIONAL**
- **Invitation System**: ✅ **FULLY OPERATIONAL**

## How to Apply Migrations

### Prerequisites
1. Wrangler CLI logged in: `wrangler auth login`
2. Access to production D1 database

### Apply to Production
```bash
# Apply initial schema
wrangler d1 execute prod-arb-edge --remote --file=sql/migrations/001_initial_schema.sql

# Apply indexes and data
wrangler d1 execute prod-arb-edge --remote --file=sql/migrations/002_indexes_and_data.sql
```

### Apply to Local Development
```bash
# Remove --remote flag for local development
wrangler d1 execute prod-arb-edge --file=sql/migrations/001_initial_schema.sql
wrangler d1 execute prod-arb-edge --file=sql/migrations/002_indexes_and_data.sql
```

### Verify Migration
```bash
# Check tables exist
wrangler d1 execute prod-arb-edge --remote --file=sql/001.check_tables.sql
```

## Creating New Migrations

When creating new migrations, follow this naming convention:
- `sql/migrations/###_descriptive_name.sql`
- Always include migration tracking:
  ```sql
  INSERT INTO schema_migrations (version, description) VALUES ('###', 'Description');
  ```

## RBAC Database Schema

### Critical RBAC Tables
1. **user_profiles**: Core user identity and subscription tiers
2. **user_trading_preferences**: Feature access control flags
3. **telegram_group_registrations**: Group-level permissions
4. **audit_log**: Security event tracking

### Permission Model
- **Subscription-based**: Free, Basic, Premium, Pro tiers
- **Feature flags**: Per-user access control (arbitrage_enabled, technical_enabled, etc.)
- **Account status**: Active, suspended, pending, deactivated

## Security Features
- Audit trail for all critical actions
- User activity tracking for permission decisions
- Role-based feature access control
- Group-level permission management

## Database Views
- `active_users`: Users with activity metrics for RBAC decisions
- `recent_opportunities`: 24-hour opportunity analytics

## Notes
- All migrations are tracked in `schema_migrations` table
- Foreign key constraints ensure data integrity
- Indexes optimized for RBAC permission checking
- Views provide efficient RBAC query patterns 