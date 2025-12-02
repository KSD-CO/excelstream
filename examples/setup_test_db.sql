-- PostgreSQL Test Database Setup SQL
-- Run this file to create the test database and populate it with sample data
-- Usage: psql -U postgres -f setup_test_db.sql

-- Create database
DROP DATABASE IF EXISTS testdb;
CREATE DATABASE testdb;

-- Connect to the new database
\c testdb

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100),
    age INTEGER,
    city VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX idx_users_age ON users(age);
CREATE INDEX idx_users_city ON users(city);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Insert sample data
-- Change the number in generate_series() to adjust the number of rows
-- Options: 1000, 10000, 100000, 1000000

\echo 'Inserting sample data...'

INSERT INTO users (name, email, age, city)
SELECT 
    'User' || i,
    'user' || i || '@example.com',
    20 + (i % 50),
    'City' || (i % 100)
FROM generate_series(1, 100000) AS i;  -- Change this number for more/fewer rows

-- Verify data
\echo 'Data inserted successfully!'
\echo ''
\echo 'Statistics:'

SELECT 
    COUNT(*) as total_rows,
    MIN(age) as min_age,
    MAX(age) as max_age,
    COUNT(DISTINCT city) as unique_cities,
    MIN(created_at) as earliest_record,
    MAX(created_at) as latest_record
FROM users;

\echo ''
\echo 'Sample data:'
SELECT * FROM users LIMIT 5;

\echo ''
\echo 'Setup complete! You can now run the examples:'
\echo '  cargo run --example postgres_to_excel --features postgres'
\echo '  cargo run --example postgres_streaming --features postgres'
\echo '  cargo run --example postgres_to_excel_advanced --features postgres-async'
