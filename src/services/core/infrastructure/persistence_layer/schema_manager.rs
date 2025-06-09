//! Schema Manager
//!
//! Comprehensive database schema management for both D1 structured data and R2 blob storage
//! with table definitions, relationships, indexes, and data type mappings

use crate::utils::{ArbitrageError, ArbitrageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaType {
    D1, // Structured data
    R2, // Blob storage
}

/// Schema version for migration tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Table definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub indexes: Vec<IndexDefinition>,
    pub constraints: Vec<ConstraintDefinition>,
    pub description: String,
    pub created_version: SchemaVersion,
    pub last_modified_version: SchemaVersion,
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub foreign_key_reference: Option<ForeignKeyReference>,
    pub description: String,
}

/// Data type mapping for cross-platform compatibility
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    BigInteger,
    Real,
    Text,
    DateTime,
    Timestamp,
    Boolean,
    Json,
    Blob,
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub description: String,
}

/// Constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintDefinition {
    pub name: String,
    pub table: String,
    pub constraint_type: ConstraintType,
    pub columns: Vec<String>,
    pub check_expression: Option<String>,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    PrimaryKey,
    Check,
    Unique,
}

/// Foreign key reference (simplified for Task 15.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyReference {
    pub table: String,
    pub column: String,
}

// R2 blob schema definitions will be added in subsequent subtasks

/// Schema manager implementation (simplified for Task 15.1)
pub struct SchemaManager {
    /// D1 table definitions
    d1_tables: HashMap<String, TableDefinition>,
    /// Current schema version
    #[allow(dead_code)]
    current_version: SchemaVersion,
    /// Data type mappings for different databases
    type_mappings: HashMap<String, HashMap<DataType, String>>,
}

impl SchemaManager {
    /// Create new schema manager
    pub async fn new(
        _d1_config: &crate::services::core::infrastructure::persistence_layer::D1Config,
        _r2_config: &crate::services::core::infrastructure::persistence_layer::R2Config,
    ) -> ArbitrageResult<Self> {
        let mut schema_manager = Self {
            d1_tables: HashMap::new(),
            current_version: SchemaVersion::new(1, 0, 0),
            type_mappings: HashMap::new(),
        };

        // Initialize D1 schemas
        schema_manager.initialize_d1_schemas().await?;

        // Initialize type mappings
        schema_manager.initialize_type_mappings().await?;

        Ok(schema_manager)
    }

    /// Initialize the schema in the database
    pub async fn initialize_schema(&self) -> ArbitrageResult<()> {
        // This would create tables if they don't exist
        // For now, we'll just validate the schema
        self.validate_schema().await?;
        Ok(())
    }

    /// Get table definition by name
    pub fn get_table_definition(&self, table_name: &str) -> Option<&TableDefinition> {
        self.d1_tables.get(table_name)
    }

    /// Get all table definitions
    pub fn get_all_tables(&self) -> &HashMap<String, TableDefinition> {
        &self.d1_tables
    }

    // R2 schema methods will be added in subsequent subtasks

    /// Generate SQL DDL for a table
    pub fn generate_table_ddl(
        &self,
        table_name: &str,
        database_type: &str,
    ) -> ArbitrageResult<String> {
        let table = self.d1_tables.get(table_name).ok_or_else(|| {
            ArbitrageError::database_error(format!("Table '{}' not found", table_name))
        })?;

        let mut ddl = format!("CREATE TABLE {} (\n", table.name);

        // Add columns
        for (i, column) in table.columns.iter().enumerate() {
            let column_sql = self.generate_column_ddl(column, database_type)?;
            ddl.push_str(&format!("    {}", column_sql));
            if i < table.columns.len() - 1 {
                ddl.push(',');
            }
            ddl.push('\n');
        }

        // Add table constraints
        for constraint in &table.constraints {
            ddl.push_str(&format!(
                "    ,{}\n",
                self.generate_constraint_ddl(constraint, database_type)?
            ));
        }

        ddl.push_str(");");
        Ok(ddl)
    }

    /// Generate index DDL
    pub fn generate_index_ddl(
        &self,
        index: &IndexDefinition,
        _database_type: &str,
    ) -> ArbitrageResult<String> {
        let unique_keyword = if index.is_unique { "UNIQUE " } else { "" };
        let columns = index.columns.join(", ");

        let ddl = format!(
            "CREATE {}INDEX {} ON {} ({});",
            unique_keyword, index.name, index.table, columns
        );

        Ok(ddl)
    }

    /// Validate schema consistency
    pub async fn validate_schema(&self) -> ArbitrageResult<Vec<String>> {
        let mut issues = Vec::new();

        // Check foreign key references
        for table in self.d1_tables.values() {
            for column in &table.columns {
                if let Some(fk_ref) = &column.foreign_key_reference {
                    if !self.d1_tables.contains_key(&fk_ref.table) {
                        issues.push(format!(
                            "Foreign key reference in table '{}' column '{}' references non-existent table '{}'",
                            table.name, column.name, fk_ref.table
                        ));
                    }
                }
            }
        }

        // Additional constraint validation can be added here

        Ok(issues)
    }

    /// Initialize D1 table schemas
    async fn initialize_d1_schemas(&mut self) -> ArbitrageResult<()> {
        // User Profiles table
        self.d1_tables.insert(
            "user_profiles".to_string(),
            TableDefinition {
                name: "user_profiles".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "user_id".to_string(),
                        data_type: DataType::Text,
                        nullable: false,
                        default_value: None,
                        is_primary_key: true,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Unique user identifier".to_string(),
                    },
                    ColumnDefinition {
                        name: "telegram_id".to_string(),
                        data_type: DataType::BigInteger,
                        nullable: false,
                        default_value: None,
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Telegram user ID".to_string(),
                    },
                    ColumnDefinition {
                        name: "username".to_string(),
                        data_type: DataType::Text,
                        nullable: true,
                        default_value: None,
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "User's username".to_string(),
                    },
                    ColumnDefinition {
                        name: "subscription_tier".to_string(),
                        data_type: DataType::Text,
                        nullable: false,
                        default_value: Some("'free'".to_string()),
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "User's subscription tier".to_string(),
                    },
                    ColumnDefinition {
                        name: "account_status".to_string(),
                        data_type: DataType::Text,
                        nullable: false,
                        default_value: Some("'active'".to_string()),
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Account status".to_string(),
                    },
                    ColumnDefinition {
                        name: "api_keys".to_string(),
                        data_type: DataType::Json,
                        nullable: true,
                        default_value: None,
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Encrypted API keys JSON".to_string(),
                    },
                    ColumnDefinition {
                        name: "created_at".to_string(),
                        data_type: DataType::DateTime,
                        nullable: false,
                        default_value: Some("datetime('now')".to_string()),
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Creation timestamp".to_string(),
                    },
                    ColumnDefinition {
                        name: "updated_at".to_string(),
                        data_type: DataType::DateTime,
                        nullable: false,
                        default_value: Some("datetime('now')".to_string()),
                        is_primary_key: false,
                        is_foreign_key: false,
                        foreign_key_reference: None,
                        description: "Last update timestamp".to_string(),
                    },
                ],
                indexes: vec![
                    IndexDefinition {
                        name: "idx_user_profiles_telegram_id".to_string(),
                        table: "user_profiles".to_string(),
                        columns: vec!["telegram_id".to_string()],
                        is_unique: true,
                        description: "Unique index on telegram_id".to_string(),
                    },
                    IndexDefinition {
                        name: "idx_user_profiles_status".to_string(),
                        table: "user_profiles".to_string(),
                        columns: vec!["account_status".to_string()],
                        is_unique: false,
                        description: "Index on account status for filtering".to_string(),
                    },
                ],
                constraints: vec![
                    ConstraintDefinition {
                        name: "pk_user_profiles".to_string(),
                        table: "user_profiles".to_string(),
                        constraint_type: ConstraintType::PrimaryKey,
                        columns: vec!["user_id".to_string()],
                        check_expression: None,
                    },
                    ConstraintDefinition {
                        name: "chk_subscription_tier".to_string(),
                        table: "user_profiles".to_string(),
                        constraint_type: ConstraintType::Check,
                        columns: vec!["subscription_tier".to_string()],
                        check_expression: Some(
                            "subscription_tier IN ('free', 'basic', 'premium', 'pro')".to_string(),
                        ),
                    },
                ],
                description: "Core user profile information".to_string(),
                created_version: SchemaVersion::new(1, 0, 0),
                last_modified_version: SchemaVersion::new(1, 0, 0),
            },
        );

        // Add more tables (opportunities, positions, trading_analytics, etc.)
        self.add_opportunities_table();
        self.add_positions_table();
        self.add_trading_analytics_table();
        self.add_system_config_table();

        Ok(())
    }

    // R2 blob schema initialization will be added in subsequent subtasks

    /// Initialize type mappings for different databases
    async fn initialize_type_mappings(&mut self) -> ArbitrageResult<()> {
        let mut sqlite_mappings = HashMap::new();
        sqlite_mappings.insert(DataType::Integer, "INTEGER".to_string());
        sqlite_mappings.insert(DataType::BigInteger, "INTEGER".to_string());
        sqlite_mappings.insert(DataType::Real, "REAL".to_string());
        sqlite_mappings.insert(DataType::Text, "TEXT".to_string());
        sqlite_mappings.insert(DataType::DateTime, "TEXT".to_string());
        sqlite_mappings.insert(DataType::Timestamp, "TEXT".to_string());
        sqlite_mappings.insert(DataType::Boolean, "INTEGER".to_string());
        sqlite_mappings.insert(DataType::Json, "TEXT".to_string());
        sqlite_mappings.insert(DataType::Blob, "BLOB".to_string());

        self.type_mappings
            .insert("sqlite".to_string(), sqlite_mappings);

        // Add PostgreSQL mappings for future compatibility
        let mut postgres_mappings = HashMap::new();
        postgres_mappings.insert(DataType::Integer, "INTEGER".to_string());
        postgres_mappings.insert(DataType::BigInteger, "BIGINT".to_string());
        postgres_mappings.insert(DataType::Real, "REAL".to_string());
        postgres_mappings.insert(DataType::Text, "TEXT".to_string());
        postgres_mappings.insert(DataType::DateTime, "TIMESTAMP".to_string());
        postgres_mappings.insert(DataType::Boolean, "BOOLEAN".to_string());
        postgres_mappings.insert(DataType::Json, "JSONB".to_string());
        postgres_mappings.insert(DataType::Blob, "BYTEA".to_string());

        self.type_mappings
            .insert("postgresql".to_string(), postgres_mappings);

        Ok(())
    }

    /// Generate column DDL
    fn generate_column_ddl(
        &self,
        column: &ColumnDefinition,
        database_type: &str,
    ) -> ArbitrageResult<String> {
        let type_mapping = self.type_mappings.get(database_type).ok_or_else(|| {
            ArbitrageError::database_error(format!("Unknown database type: {}", database_type))
        })?;

        let sql_type = type_mapping.get(&column.data_type).ok_or_else(|| {
            ArbitrageError::database_error(format!("Unsupported data type: {:?}", column.data_type))
        })?;

        let mut column_sql = format!("{} {}", column.name, sql_type);

        if !column.nullable {
            column_sql.push_str(" NOT NULL");
        }

        if let Some(default) = &column.default_value {
            column_sql.push_str(&format!(" DEFAULT {}", default));
        }

        Ok(column_sql)
    }

    /// Generate constraint DDL
    fn generate_constraint_ddl(
        &self,
        constraint: &ConstraintDefinition,
        _database_type: &str,
    ) -> ArbitrageResult<String> {
        match constraint.constraint_type {
            ConstraintType::PrimaryKey => Ok(format!(
                "CONSTRAINT {} PRIMARY KEY ({})",
                constraint.name,
                constraint.columns.join(", ")
            )),
            ConstraintType::Check => {
                let check_expr = constraint.check_expression.as_ref().ok_or_else(|| {
                    ArbitrageError::database_error("Check constraint missing expression")
                })?;

                Ok(format!(
                    "CONSTRAINT {} CHECK ({})",
                    constraint.name, check_expr
                ))
            }
            ConstraintType::Unique => Ok(format!(
                "CONSTRAINT {} UNIQUE ({})",
                constraint.name,
                constraint.columns.join(", ")
            )),
        }
    }

    /// Add opportunities table definition
    fn add_opportunities_table(&mut self) {
        // Implementation for opportunities table...
        // This would be similar to user_profiles but with opportunity-specific columns
    }

    /// Add positions table definition
    fn add_positions_table(&mut self) {
        // Implementation for positions table...
    }

    /// Add trading analytics table definition
    fn add_trading_analytics_table(&mut self) {
        // Implementation for trading analytics table...
    }

    /// Add system config table definition
    fn add_system_config_table(&mut self) {
        // Implementation for system config table...
    }
}
