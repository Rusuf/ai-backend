-- Dummy data for the ai_backend PostgreSQL database

-- Insert sample products
INSERT INTO products (product_id, code, name, category, department, unit, buy_price, sell_price, current_stock, updated_at) VALUES
('prod_1', 'P001', 'Cafe Latte', 'Hot Beverages', 'Coffee', 'Cup', 1.50, 3.50, 100, NOW()),
('prod_2', 'P002', 'Croissant', 'Pastries', 'Bakery', 'Piece', 0.80, 2.50, 50, NOW()),
('prod_3', 'P003', 'Iced Tea', 'Cold Beverages', 'Drinks', 'Glass', 0.50, 2.00, 75, NOW());

-- Insert a sample receipt
INSERT INTO receipts (receipt_id, receipt_no, at, payment_channel, customer) VALUES
(1001, 5001, NOW() - INTERVAL '1 day', 'Cash', 'John Doe');

-- Insert sample sales that link to the products and the receipt
INSERT INTO sales (sale_id, at, receipt_no, product_code, product_name, qty, price, total, customer) VALUES
(2001, NOW() - INTERVAL '1 day', 5001, 'P001', 'Cafe Latte', 1, 3.50, 3.50, 'John Doe'),
(2002, NOW() - INTERVAL '1 day', 5001, 'P002', 'Croissant', 2, 2.50, 5.00, 'John Doe');

-- Insert a second sample receipt
INSERT INTO receipts (receipt_id, receipt_no, at, payment_channel, customer) VALUES
(1002, 5002, NOW(), 'Credit Card', 'Jane Smith');

-- Insert a sale for the second receipt
INSERT INTO sales (sale_id, at, receipt_no, product_code, product_name, qty, price, total, customer) VALUES
(2003, NOW(), 5002, 'P003', 'Iced Tea', 1, 2.00, 2.00, 'Jane Smith');

-- Set an initial watermark for the ingest process
INSERT INTO ingest_state (id, watermark) VALUES
('main', NOW() - INTERVAL '2 days');
