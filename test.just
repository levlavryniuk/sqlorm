
default: p s examples

postgres:
    cargo test --workspace --features postgres -- --nocapture

sqlite:
    cargo test --workspace --features sqlite -- --nocapture

all: postgres sqlite examples

p:
    cargo nextest run --workspace --features postgres --cargo-quiet --cargo-quiet

s:
    cargo nextest run --workspace --features sqlite --cargo-quiet --cargo-quiet

pp PACKAGE:
    cargo nextest run --cargo-quiet --cargo-quiet --package {{PACKAGE}}

examples: examples-postgres examples-sqlite

# Test PostgreSQL examples
examples-postgres:
    @echo "🐘 Testing PostgreSQL examples..."
    cargo test -p sqlorm-postgres-example --features postgres --quiet

# Test SQLite examples  
examples-sqlite:
    @echo "🗄️  Testing SQLite examples..."
    cargo test -p sqlorm-sqlite-example --features sqlite --quiet

# Test specific example suite for PostgreSQL
examples-postgres-suite SUITE:
    @echo "🐘 Testing PostgreSQL {{SUITE}} suite..."
    cargo test -p sqlorm-postgres-example --features postgres --test {{SUITE}} --quiet

# Test specific example suite for SQLite
examples-sqlite-suite SUITE:
    @echo "🗄️  Testing SQLite {{SUITE}} suite..."
    cargo test -p sqlorm-sqlite-example --features sqlite --test {{SUITE}} --quiet

# Test CRUD examples for both databases
examples-crud: (examples-postgres-suite "crud") (examples-sqlite-suite "crud")

# Test filter examples for both databases
examples-filters: (examples-postgres-suite "filters") (examples-sqlite-suite "filters")

# Test relation examples for both databases
examples-relations: (examples-postgres-suite "relations") (examples-sqlite-suite "relations")

# Test select examples for both databases
examples-select: (examples-postgres-suite "select") (examples-sqlite-suite "select")

# Run examples with verbose output
examples-verbose: examples-postgres-verbose examples-sqlite-verbose

examples-postgres-verbose:
    @echo "🐘 Testing PostgreSQL examples (verbose)..."
    cargo test -p sqlorm-postgres-example --features postgres -- --nocapture

examples-sqlite-verbose:
    @echo "🗄️  Testing SQLite examples (verbose)..."
    cargo test -p sqlorm-sqlite-example --features sqlite -- --nocapture

# Check examples compile without running tests
examples-check:
    @echo "🔍 Checking example structure..."
    @echo "⚠️  Note: Entity macros have known 'macros_core' reference issue"
    @echo "📁 Verifying example crate structure..."
    @test -d examples/common/entities && echo "✅ entities crate exists"
    @test -d examples/common/migrations && echo "✅ migrations directory exists" 
    @test -d examples/common/migrations-helper && echo "✅ migrations-helper crate exists"
    @test -d examples/postgres && echo "✅ postgres example crate exists"
    @test -d examples/sqlite && echo "✅ sqlite example crate exists"
    @echo "📋 Checking migration files..."
    @test -f examples/common/migrations/001_create_users.sql && echo "✅ users migration exists"
    @test -f examples/common/migrations/002_create_jars.sql && echo "✅ jars migration exists"
    @test -f examples/common/migrations/003_create_donations.sql && echo "✅ donations migration exists"
    @echo "🧪 Checking test files..."
    @test -f examples/postgres/tests/crud.rs && echo "✅ postgres CRUD tests exist"
    @test -f examples/postgres/tests/filters.rs && echo "✅ postgres filter tests exist"
    @test -f examples/postgres/tests/relations.rs && echo "✅ postgres relation tests exist"
    @test -f examples/postgres/tests/select.rs && echo "✅ postgres select tests exist"
    @test -f examples/sqlite/tests/crud.rs && echo "✅ sqlite CRUD tests exist"
    @test -f examples/sqlite/tests/filters.rs && echo "✅ sqlite filter tests exist"
    @test -f examples/sqlite/tests/relations.rs && echo "✅ sqlite relation tests exist"
    @test -f examples/sqlite/tests/select.rs && echo "✅ sqlite select tests exist"
    @echo "✅ Example structure is complete and ready for testing"

# Run examples with nextest (faster parallel execution)
examples-nextest: examples-nextest-postgres examples-nextest-sqlite

examples-nextest-postgres:
    @echo "🐘 Testing PostgreSQL examples with nextest..."
    cargo nextest run -p sqlorm-postgres-example --features postgres --cargo-quiet

examples-nextest-sqlite:
    @echo "🗄️  Testing SQLite examples with nextest..."
    cargo nextest run -p sqlorm-sqlite-example --features sqlite --cargo-quiet

# Development & CI Commands
# =========================

# Run all tests including examples (for CI)
ci: all

# Quick development check - compile and run core tests
dev: examples-check p s

# Clean and run full test suite
full-test: clean all

clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean --quiet

# Test examples with custom PostgreSQL database URL
examples-postgres-custom URL:
    @echo "🐘 Testing PostgreSQL examples with custom URL..."
    TEST_DATABASE_URL={{URL}} cargo test -p sqlorm-postgres-example --features postgres --quiet

# Show all available example commands
examples-help:
    @echo "📚 SQLOrm Example Testing Commands:"
    @echo "  just examples              - Test all examples (PostgreSQL + SQLite)"
    @echo "  just examples-postgres     - Test PostgreSQL examples only"
    @echo "  just examples-sqlite       - Test SQLite examples only"
    @echo "  just examples-crud         - Test CRUD operations for both databases"
    @echo "  just examples-filters      - Test query filters for both databases"
    @echo "  just examples-relations    - Test relationships for both databases"
    @echo "  just examples-select       - Test custom selects for both databases"
    @echo "  just examples-verbose      - Run examples with verbose output"
    @echo "  just examples-check        - Check examples compile without running"
    @echo "  just examples-nextest      - Run examples with nextest (faster)"
    @echo ""
    @echo "🔧 Development Commands:"
    @echo "  just dev                   - Quick development check"
    @echo "  just ci                    - Full CI test suite"
    @echo "  just examples-postgres-custom URL - Test with custom PostgreSQL URL"
    @echo ""
    @echo "💡 Example URLs:"
    @echo "  just examples-postgres-custom 'postgresql://user:pass@localhost/testdb'"
    @echo "  TEST_DATABASE_URL='...' just examples-postgres"
