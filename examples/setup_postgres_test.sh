#!/bin/bash
# PostgreSQL Test Database Setup Script
# This script creates a test database and populates it with sample data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== PostgreSQL Test Database Setup ===${NC}\n"

# Configuration
DB_HOST=${DB_HOST:-localhost}
DB_PORT=${DB_PORT:-5432}
DB_USER=${DB_USER:-postgres}
DB_NAME=${DB_NAME:-testdb}
DB_ADMIN=${DB_ADMIN:-postgres}

# Ask for number of rows
echo -e "${YELLOW}How many sample rows do you want to insert?${NC}"
echo "  1) 1,000 rows (small test)"
echo "  2) 10,000 rows (medium test)"
echo "  3) 100,000 rows (large test)"
echo "  4) 1,000,000 rows (very large test)"
echo "  5) Custom amount"
read -p "Enter choice [1-5]: " choice

case $choice in
    1) ROWS=1000 ;;
    2) ROWS=10000 ;;
    3) ROWS=100000 ;;
    4) ROWS=1000000 ;;
    5) 
        read -p "Enter number of rows: " ROWS
        ;;
    *)
        echo -e "${RED}Invalid choice. Using 1,000 rows.${NC}"
        ROWS=1000
        ;;
esac

echo -e "\n${GREEN}Configuration:${NC}"
echo "  Host: $DB_HOST"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"
echo "  Admin User: $DB_ADMIN"
echo "  Sample Rows: $ROWS"
echo ""

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo -e "${RED}Error: psql command not found.${NC}"
    echo "Please install PostgreSQL client tools."
    exit 1
fi

echo -e "${YELLOW}Please enter the PostgreSQL admin password when prompted.${NC}\n"

# Create database
echo -e "${GREEN}Step 1: Creating database...${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_ADMIN" -c "DROP DATABASE IF EXISTS $DB_NAME;" postgres 2>/dev/null || true
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_ADMIN" -c "CREATE DATABASE $DB_NAME;" postgres

# Create table
echo -e "${GREEN}Step 2: Creating users table...${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_ADMIN" -d "$DB_NAME" << EOF
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100),
    age INTEGER,
    city VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_age ON users(age);
CREATE INDEX idx_users_city ON users(city);
CREATE INDEX idx_users_created_at ON users(created_at);
EOF

# Insert sample data
echo -e "${GREEN}Step 3: Inserting $ROWS sample rows...${NC}"
echo -e "${YELLOW}This may take a while for large datasets...${NC}"

START_TIME=$(date +%s)

psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_ADMIN" -d "$DB_NAME" << EOF
INSERT INTO users (name, email, age, city)
SELECT 
    'User' || i,
    'user' || i || '@example.com',
    20 + (i % 50),
    'City' || (i % 100)
FROM generate_series(1, $ROWS) AS i;
EOF

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Verify
echo -e "${GREEN}Step 4: Verifying data...${NC}"
ROW_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_ADMIN" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM users;")

echo -e "\n${GREEN}=== Setup Complete ===${NC}"
echo -e "Database: ${GREEN}$DB_NAME${NC}"
echo -e "Table: ${GREEN}users${NC}"
echo -e "Rows inserted: ${GREEN}$(echo $ROW_COUNT | xargs)${NC}"
echo -e "Time taken: ${GREEN}${DURATION}s${NC}"

# Export connection string
export DATABASE_URL="postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"

echo -e "\n${YELLOW}Connection string:${NC}"
echo "  export DATABASE_URL=\"postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME\""

echo -e "\n${GREEN}Sample queries:${NC}"
echo "  # View sample data"
echo "  psql -h $DB_HOST -p $DB_PORT -U $DB_ADMIN -d $DB_NAME -c \"SELECT * FROM users LIMIT 10;\""
echo ""
echo "  # View statistics"
echo "  psql -h $DB_HOST -p $DB_PORT -U $DB_ADMIN -d $DB_NAME -c \"SELECT COUNT(*) as total, MIN(age) as min_age, MAX(age) as max_age FROM users;\""

echo -e "\n${GREEN}Now you can run the examples:${NC}"
echo "  cargo run --example postgres_to_excel --features postgres"
echo "  cargo run --example postgres_streaming --features postgres"
echo "  cargo run --example postgres_to_excel_advanced --features postgres-async"

echo -e "\n${GREEN}✓ All done!${NC}"
