DROP SCHEMA public CASCADE;
CREATE SCHEMA public;

CREATE TABLE products (
    product_id INTEGER PRIMARY KEY,
    product_code VARCHAR(100) UNIQUE,
    name VARCHAR(100),
    department VARCHAR(100),
    category VARCHAR(100),
    selling_price REAL,
    current_stock REAL
);

CREATE TABLE customers (
    customer_id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100),
    registered_on TIMESTAMP
);

CREATE TABLE receipts (
    receipt_id INTEGER PRIMARY KEY,
    receipt_no INTEGER,
    transaction_date TIMESTAMP,
    customer_id INTEGER REFERENCES customers(customer_id),
    total_amount REAL,
    payment_channel VARCHAR(100)
);

CREATE TABLE sales (
    sale_id INTEGER PRIMARY KEY,
    receipt_id INTEGER REFERENCES receipts(receipt_id),
    product_id INTEGER REFERENCES products(product_id),
    quantity REAL,
    selling_price REAL,
    total_sale REAL
);