-- Add migration script here
CREATE TABLE IF NOT EXISTS customers (
    customer_id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    registered_on TIMESTAMP
);

CREATE TABLE IF NOT EXISTS products (
    product_id INTEGER PRIMARY KEY,
    product_code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    department VARCHAR(100),
    category VARCHAR(100),
    selling_price DECIMAL(10, 2),
    current_stock DECIMAL(10, 2)
);

CREATE TABLE IF NOT EXISTS receipts (
    receipt_id INTEGER PRIMARY KEY,
    receipt_no INTEGER NOT NULL UNIQUE,
    transaction_date TIMESTAMP,
    customer_id INTEGER REFERENCES customers(customer_id),
    total_amount DECIMAL(10, 2),
    payment_channel VARCHAR(50)
);


CREATE TABLE IF NOT EXISTS sales (
    sale_id INTEGER PRIMARY KEY,
    receipt_id INTEGER REFERENCES receipts(receipt_id),
    product_id INTEGER REFERENCES products(product_id),
    quantity DECIMAL(10, 2),
    selling_price DECIMAL(10, 2),
    total_sale DECIMAL(10, 2)
);
