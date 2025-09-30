
-- Minimal schema for the AI backend analytics store

CREATE TABLE IF NOT EXISTS products (
    product_id TEXT PRIMARY KEY,
    code TEXT,
    name TEXT,
    category TEXT,
    department TEXT,
    unit TEXT,
    buy_price NUMERIC,
    sell_price NUMERIC,
    current_stock NUMERIC,
    updated_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sales (
    sale_id BIGINT PRIMARY KEY,
    at TIMESTAMPTZ,
    receipt_no BIGINT,
    product_code TEXT,
    product_name TEXT,
    qty NUMERIC,
    price NUMERIC,
    total NUMERIC,
    customer TEXT
);

CREATE TABLE IF NOT EXISTS receipts (
    receipt_id BIGINT PRIMARY KEY,
    receipt_no BIGINT UNIQUE,
    at TIMESTAMPTZ,
    payment_channel TEXT,
    customer TEXT
);

CREATE TABLE IF NOT EXISTS ingest_state (
    id TEXT PRIMARY KEY,
    watermark TIMESTAMPTZ
);
