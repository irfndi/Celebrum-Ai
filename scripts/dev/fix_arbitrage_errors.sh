#!/bin/bash

# Fix ArbitrageError API calls to use correct method names
# This script fixes compilation errors in the circuit breaker infrastructure

set -e

echo "Fixing ArbitrageError API calls..."

# Fix system -> internal_error
echo "Replacing ArbitrageError::system with ArbitrageError::internal_error..."
find src/services/core/infrastructure/ -name "*.rs" -exec sed -i '' 's/ArbitrageError::system(/ArbitrageError::internal_error(/g' {} \;

# Fix external -> api_error  
echo "Replacing ArbitrageError::external with ArbitrageError::api_error..."
find src/services/core/infrastructure/ -name "*.rs" -exec sed -i '' 's/ArbitrageError::external(/ArbitrageError::api_error(/g' {} \;

# Fix config -> config_error
echo "Replacing ArbitrageError::config with ArbitrageError::config_error..."
find src/services/core/infrastructure/ -name "*.rs" -exec sed -i '' 's/ArbitrageError::config(/ArbitrageError::config_error(/g' {} \;

echo "Fixed ArbitrageError API calls."

# Check compilation
echo "Checking compilation..."
cargo check --lib

echo "Done!" 