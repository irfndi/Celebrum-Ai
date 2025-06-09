//! Migration Utilities
//!
//! Helper functions and utilities for database migrations including SQL generation,
//! data transformation, migration discovery, and common migration patterns.

use crate::utils::error::ArbitrageResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::migration_engine::{
    Migration, MigrationOperation, MigrationTarget, MigrationVersion, SchemaChangeType,
    ValidationCheck, ValidationResult,
};

/// Migration builder for creating migrations with fluent API
pub struct MigrationBuilder {
    migration: Migration,
}

impl MigrationBuilder {
    /// Create new migration builder
    pub fn new(id: &str, name: &str, version: MigrationVersion) -> Self {
        Self {
            migration: Migration {
                id: id.to_string(),
                name: name.to_string(),
                version,
                target: MigrationTarget::D1Database,
                up_operations: Vec::new(),
                down_operations: Vec::new(),
                dependencies: Vec::new(),
                estimated_duration_secs: 10,
                tags: Vec::new(),
                validation_checks: Vec::new(),
                created_at: Utc::now(),
                author: "system".to_string(),
            },
        }
    }

    /// Set migration target
    pub fn target(mut self, target: MigrationTarget) -> Self {
        self.migration.target = target;
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, migration_id: &str) -> Self {
        self.migration.dependencies.push(migration_id.to_string());
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: &str) -> Self {
        self.migration.tags.push(tag.to_string());
        self
    }

    /// Set author
    pub fn author(mut self, author: &str) -> Self {
        self.migration.author = author.to_string();
        self
    }

    /// Set estimated duration
    pub fn estimated_duration(mut self, seconds: u32) -> Self {
        self.migration.estimated_duration_secs = seconds;
        self
    }

    /// Add SQL operation
    pub fn add_sql(mut self, sql: &str, rollback_sql: Option<&str>) -> Self {
        let operation = MigrationOperation::SqlOperation {
            sql: sql.to_string(),
            parameters: Vec::new(),
            rollback_sql: rollback_sql.map(|s| s.to_string()),
        };
        self.migration.up_operations.push(operation);

        // Add rollback operation if provided
        if let Some(rollback) = rollback_sql {
            let rollback_operation = MigrationOperation::SqlOperation {
                sql: rollback.to_string(),
                parameters: Vec::new(),
                rollback_sql: Some(sql.to_string()),
            };
            self.migration.down_operations.push(rollback_operation);
        }

        self
    }

    /// Add create table operation
    pub fn create_table(self, table_name: &str, columns: &[ColumnDefinition]) -> Self {
        let sql = SqlGenerator::create_table(table_name, columns);
        let rollback_sql = format!("DROP TABLE IF EXISTS {}", table_name);
        self.add_sql(&sql, Some(&rollback_sql))
    }

    /// Add drop table operation
    pub fn drop_table(self, table_name: &str) -> Self {
        let sql = format!("DROP TABLE IF EXISTS {}", table_name);
        // Note: Cannot generate CREATE TABLE rollback without schema info
        self.add_sql(&sql, None)
    }

    /// Add create index operation
    pub fn create_index(
        self,
        index_name: &str,
        table_name: &str,
        columns: &[&str],
        unique: bool,
    ) -> Self {
        let sql = SqlGenerator::create_index(index_name, table_name, columns, unique);
        let rollback_sql = format!("DROP INDEX IF EXISTS {}", index_name);
        self.add_sql(&sql, Some(&rollback_sql))
    }

    /// Add validation check
    pub fn validate_with(mut self, name: &str, description: &str, query: &str) -> Self {
        let check = ValidationCheck {
            name: name.to_string(),
            description: description.to_string(),
            check_query: query.to_string(),
            expected_result: ValidationResult::AnyRows,
        };
        self.migration.validation_checks.push(check);
        self
    }

    /// Build the migration
    pub fn build(self) -> Migration {
        self.migration
    }
}

/// Column definition for SQL generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
    pub foreign_key: Option<ForeignKeyDefinition>,
}

/// Foreign key definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyDefinition {
    pub table: String,
    pub column: String,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

impl ColumnDefinition {
    /// Create new column definition
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: true,
            default_value: None,
            primary_key: false,
            unique: false,
            auto_increment: false,
            foreign_key: None,
        }
    }

    /// Make column not nullable
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Set default value
    pub fn default(mut self, value: &str) -> Self {
        self.default_value = Some(value.to_string());
        self
    }

    /// Make column primary key
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    /// Make column unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Make column auto increment
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self.nullable = false;
        self
    }

    /// Add foreign key constraint
    pub fn foreign_key(mut self, table: &str, column: &str) -> Self {
        self.foreign_key = Some(ForeignKeyDefinition {
            table: table.to_string(),
            column: column.to_string(),
            on_delete: None,
            on_update: None,
        });
        self
    }
}

/// SQL generation utilities
pub struct SqlGenerator;

impl SqlGenerator {
    /// Generate CREATE TABLE SQL
    pub fn create_table(table_name: &str, columns: &[ColumnDefinition]) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", table_name);

        let column_definitions: Vec<String> =
            columns.iter().map(Self::column_definition_sql).collect();

        sql.push_str(&column_definitions.join(",\n"));

        // Add constraints
        let constraints = Self::generate_constraints(columns);
        if !constraints.is_empty() {
            sql.push_str(",\n");
            sql.push_str(&constraints.join(",\n"));
        }

        sql.push_str("\n)");
        sql
    }

    /// Generate CREATE INDEX SQL
    pub fn create_index(
        index_name: &str,
        table_name: &str,
        columns: &[&str],
        unique: bool,
    ) -> String {
        let unique_keyword = if unique { "UNIQUE " } else { "" };
        format!(
            "CREATE {}INDEX {} ON {} ({})",
            unique_keyword,
            index_name,
            table_name,
            columns.join(", ")
        )
    }

    /// Generate ALTER TABLE ADD COLUMN SQL
    pub fn add_column(table_name: &str, column: &ColumnDefinition) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {}",
            table_name,
            Self::column_definition_sql(column)
        )
    }

    /// Generate ALTER TABLE DROP COLUMN SQL
    pub fn drop_column(table_name: &str, column_name: &str) -> String {
        format!("ALTER TABLE {} DROP COLUMN {}", table_name, column_name)
    }

    /// Generate column definition SQL
    fn column_definition_sql(column: &ColumnDefinition) -> String {
        let mut definition = format!("  {} {}", column.name, column.data_type);

        if column.primary_key {
            definition.push_str(" PRIMARY KEY");
        }

        if column.auto_increment {
            definition.push_str(" AUTOINCREMENT");
        }

        if !column.nullable && !column.primary_key {
            definition.push_str(" NOT NULL");
        }

        if column.unique && !column.primary_key {
            definition.push_str(" UNIQUE");
        }

        if let Some(ref default) = column.default_value {
            definition.push_str(&format!(" DEFAULT {}", default));
        }

        definition
    }

    /// Generate constraints SQL
    fn generate_constraints(columns: &[ColumnDefinition]) -> Vec<String> {
        let mut constraints = Vec::new();

        for column in columns {
            if let Some(ref fk) = column.foreign_key {
                let mut fk_constraint = format!(
                    "  FOREIGN KEY ({}) REFERENCES {} ({})",
                    column.name, fk.table, fk.column
                );

                if let Some(ref on_delete) = fk.on_delete {
                    fk_constraint.push_str(&format!(" ON DELETE {}", on_delete));
                }

                if let Some(ref on_update) = fk.on_update {
                    fk_constraint.push_str(&format!(" ON UPDATE {}", on_update));
                }

                constraints.push(fk_constraint);
            }
        }

        constraints
    }
}

/// Data transformation utilities
pub struct DataTransformer;

impl DataTransformer {
    /// Generate SQL for data migration between tables
    pub fn migrate_data(
        source_table: &str,
        target_table: &str,
        column_mapping: &HashMap<String, String>,
        where_clause: Option<&str>,
    ) -> String {
        let columns: Vec<String> = column_mapping
            .iter()
            .map(|(source, target)| {
                if source == target {
                    source.clone()
                } else {
                    format!("{} AS {}", source, target)
                }
            })
            .collect();

        let mut sql = format!(
            "INSERT INTO {} SELECT {} FROM {}",
            target_table,
            columns.join(", "),
            source_table
        );

        if let Some(where_clause) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_clause));
        }

        sql
    }

    /// Generate batch update SQL
    pub fn batch_update(
        table_name: &str,
        set_clause: &str,
        where_clause: &str,
        batch_size: u32,
    ) -> String {
        format!(
            "UPDATE {} SET {} WHERE {} LIMIT {}",
            table_name, set_clause, where_clause, batch_size
        )
    }

    /// Generate data cleanup SQL
    pub fn cleanup_orphaned_records(
        table_name: &str,
        foreign_key_column: &str,
        referenced_table: &str,
        referenced_column: &str,
    ) -> String {
        format!(
            "DELETE FROM {} WHERE {} NOT IN (SELECT {} FROM {})",
            table_name, foreign_key_column, referenced_column, referenced_table
        )
    }
}

/// Migration discovery utilities
pub struct MigrationDiscovery;

impl MigrationDiscovery {
    /// Discover schema differences and generate migration operations
    pub fn discover_schema_changes(
        current_schema: &SchemaSnapshot,
        target_schema: &SchemaSnapshot,
    ) -> ArbitrageResult<Vec<MigrationOperation>> {
        let mut operations = Vec::new();

        // Discover new tables
        for (table_name, table_def) in &target_schema.tables {
            if !current_schema.tables.contains_key(table_name) {
                operations.push(MigrationOperation::SchemaChange {
                    change_type: SchemaChangeType::CreateTable,
                    table_name: table_name.clone(),
                    definition: serde_json::to_value(table_def)?,
                });
            }
        }

        // Discover dropped tables
        for table_name in current_schema.tables.keys() {
            if !target_schema.tables.contains_key(table_name) {
                operations.push(MigrationOperation::SchemaChange {
                    change_type: SchemaChangeType::DropTable,
                    table_name: table_name.clone(),
                    definition: serde_json::Value::Null,
                });
            }
        }

        // Discover new indexes
        for (index_name, index_def) in &target_schema.indexes {
            if !current_schema.indexes.contains_key(index_name) {
                operations.push(MigrationOperation::SchemaChange {
                    change_type: SchemaChangeType::CreateIndex,
                    table_name: index_def.table_name.clone(),
                    definition: serde_json::to_value(index_def)?,
                });
            }
        }

        Ok(operations)
    }

    /// Generate automatic migration from schema differences
    pub fn generate_auto_migration(
        current_schema: &SchemaSnapshot,
        target_schema: &SchemaSnapshot,
        version: MigrationVersion,
    ) -> ArbitrageResult<Migration> {
        let operations = Self::discover_schema_changes(current_schema, target_schema)?;

        let migration_id = Uuid::new_v4().to_string();
        let migration = Migration {
            id: migration_id,
            name: "Auto-generated schema migration".to_string(),
            version,
            target: MigrationTarget::D1Database,
            up_operations: operations,
            down_operations: Vec::new(), // TODO: Generate reverse operations
            dependencies: Vec::new(),
            estimated_duration_secs: 30,
            tags: vec!["auto-generated".to_string()],
            validation_checks: Vec::new(),
            created_at: Utc::now(),
            author: "auto-discovery".to_string(),
        };

        Ok(migration)
    }
}

/// Schema snapshot for migration discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSnapshot {
    /// Tables in the schema
    pub tables: HashMap<String, TableSnapshot>,
    /// Indexes in the schema
    pub indexes: HashMap<String, IndexSnapshot>,
    /// Constraints in the schema
    pub constraints: HashMap<String, ConstraintSnapshot>,
    /// Schema version
    pub version: MigrationVersion,
    /// Snapshot timestamp
    pub created_at: DateTime<Utc>,
}

/// Table snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub primary_key: Vec<String>,
}

/// Index snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// Constraint snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSnapshot {
    pub name: String,
    pub table_name: String,
    pub constraint_type: String,
    pub definition: String,
}

/// Common migration patterns
pub struct CommonMigrations;

impl CommonMigrations {
    /// Create user tables migration
    pub fn create_user_tables() -> Migration {
        MigrationBuilder::new("001", "Create user tables", MigrationVersion::new(1, 0, 0))
            .tag("users")
            .tag("core")
            .create_table(
                "users",
                &[
                    ColumnDefinition::new("id", "TEXT").primary_key(),
                    ColumnDefinition::new("email", "TEXT").not_null().unique(),
                    ColumnDefinition::new("username", "TEXT")
                        .not_null()
                        .unique(),
                    ColumnDefinition::new("created_at", "INTEGER").not_null(),
                    ColumnDefinition::new("updated_at", "INTEGER").not_null(),
                ],
            )
            .create_table(
                "user_profiles",
                &[
                    ColumnDefinition::new("user_id", "TEXT").primary_key(),
                    ColumnDefinition::new("display_name", "TEXT"),
                    ColumnDefinition::new("bio", "TEXT"),
                    ColumnDefinition::new("avatar_url", "TEXT"),
                ],
            )
            .create_index("idx_users_email", "users", &["email"], true)
            .create_index("idx_users_username", "users", &["username"], true)
            .validate_with(
                "users_table_exists",
                "Verify users table was created",
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
            )
            .build()
    }

    /// Add timestamp columns to existing table
    pub fn add_timestamps(table_name: &str, migration_id: &str) -> Migration {
        MigrationBuilder::new(
            migration_id,
            &format!("Add timestamps to {}", table_name),
            MigrationVersion::new(1, 0, 1),
        )
        .tag("timestamps")
        .add_sql(
            &format!("ALTER TABLE {} ADD COLUMN created_at INTEGER", table_name),
            Some(&format!(
                "ALTER TABLE {} DROP COLUMN created_at",
                table_name
            )),
        )
        .add_sql(
            &format!("ALTER TABLE {} ADD COLUMN updated_at INTEGER", table_name),
            Some(&format!(
                "ALTER TABLE {} DROP COLUMN updated_at",
                table_name
            )),
        )
        .add_sql(
            &format!(
                "UPDATE {} SET created_at = strftime('%s', 'now'), updated_at = strftime('%s', 'now')",
                table_name
            ),
            None,
        )
        .build()
    }

    /// Create API keys table
    pub fn create_api_keys_table() -> Migration {
        MigrationBuilder::new(
            "002",
            "Create API keys table",
            MigrationVersion::new(1, 0, 2),
        )
        .tag("api")
        .tag("security")
        .create_table(
            "api_keys",
            &[
                ColumnDefinition::new("id", "TEXT").primary_key(),
                ColumnDefinition::new("user_id", "TEXT")
                    .not_null()
                    .foreign_key("users", "id"),
                ColumnDefinition::new("key_hash", "TEXT")
                    .not_null()
                    .unique(),
                ColumnDefinition::new("name", "TEXT").not_null(),
                ColumnDefinition::new("permissions", "TEXT").not_null(),
                ColumnDefinition::new("expires_at", "INTEGER"),
                ColumnDefinition::new("last_used_at", "INTEGER"),
                ColumnDefinition::new("created_at", "INTEGER").not_null(),
                ColumnDefinition::new("is_active", "INTEGER")
                    .not_null()
                    .default("1"),
            ],
        )
        .create_index("idx_api_keys_user_id", "api_keys", &["user_id"], false)
        .create_index("idx_api_keys_key_hash", "api_keys", &["key_hash"], true)
        .validate_with(
            "api_keys_foreign_key",
            "Verify foreign key constraint",
            "SELECT COUNT(*) FROM api_keys WHERE user_id NOT IN (SELECT id FROM users)",
        )
        .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_builder() {
        let migration =
            MigrationBuilder::new("test", "Test Migration", MigrationVersion::new(1, 0, 0))
                .tag("test")
                .author("test-author")
                .estimated_duration(60)
                .add_sql(
                    "CREATE TABLE test (id INTEGER PRIMARY KEY)",
                    Some("DROP TABLE test"),
                )
                .build();

        assert_eq!(migration.id, "test");
        assert_eq!(migration.name, "Test Migration");
        assert_eq!(migration.version, MigrationVersion::new(1, 0, 0));
        assert_eq!(migration.tags, vec!["test"]);
        assert_eq!(migration.author, "test-author");
        assert_eq!(migration.estimated_duration_secs, 60);
        assert_eq!(migration.up_operations.len(), 1);
        assert_eq!(migration.down_operations.len(), 1);
    }

    #[test]
    fn test_column_definition() {
        let column = ColumnDefinition::new("test_col", "TEXT")
            .not_null()
            .unique()
            .default("'default_value'");

        assert_eq!(column.name, "test_col");
        assert_eq!(column.data_type, "TEXT");
        assert!(!column.nullable);
        assert!(column.unique);
        assert_eq!(column.default_value, Some("'default_value'".to_string()));
    }

    #[test]
    fn test_sql_generator_create_table() {
        let columns = vec![
            ColumnDefinition::new("id", "INTEGER").primary_key(),
            ColumnDefinition::new("name", "TEXT").not_null(),
            ColumnDefinition::new("email", "TEXT").not_null().unique(),
        ];

        let sql = SqlGenerator::create_table("users", &columns);
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id INTEGER PRIMARY KEY"));
        assert!(sql.contains("name TEXT NOT NULL"));
        assert!(sql.contains("email TEXT NOT NULL UNIQUE"));
    }

    #[test]
    fn test_sql_generator_create_index() {
        let sql = SqlGenerator::create_index("idx_users_email", "users", &["email"], true);
        assert_eq!(sql, "CREATE UNIQUE INDEX idx_users_email ON users (email)");

        let sql = SqlGenerator::create_index("idx_users_name", "users", &["name"], false);
        assert_eq!(sql, "CREATE INDEX idx_users_name ON users (name)");
    }

    #[test]
    fn test_data_transformer_migrate_data() {
        let mut mapping = HashMap::new();
        mapping.insert("old_name".to_string(), "new_name".to_string());
        mapping.insert("id".to_string(), "id".to_string());

        let sql =
            DataTransformer::migrate_data("old_table", "new_table", &mapping, Some("active = 1"));
        assert!(sql.contains("INSERT INTO new_table"));
        assert!(sql.contains("old_name AS new_name"));
        assert!(sql.contains("WHERE active = 1"));
    }

    #[test]
    fn test_common_migrations_user_tables() {
        let migration = CommonMigrations::create_user_tables();
        assert_eq!(migration.id, "001");
        assert_eq!(migration.name, "Create user tables");
        assert!(migration.tags.contains(&"users".to_string()));
        assert!(migration.tags.contains(&"core".to_string()));
        assert!(!migration.up_operations.is_empty());
        assert!(!migration.validation_checks.is_empty());
    }

    #[test]
    fn test_common_migrations_add_timestamps() {
        let migration = CommonMigrations::add_timestamps("test_table", "003");
        assert_eq!(migration.id, "003");
        assert!(migration.name.contains("Add timestamps"));
        assert!(migration.tags.contains(&"timestamps".to_string()));
        assert_eq!(migration.up_operations.len(), 3); // Two ALTER TABLE + one UPDATE
    }
}
