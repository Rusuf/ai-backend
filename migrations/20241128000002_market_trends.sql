CREATE TABLE IF NOT EXISTS market_trends (
    id SERIAL PRIMARY KEY,
    category VARCHAR(100) NOT NULL,
    dish VARCHAR(100) NOT NULL,
    source VARCHAR(100) NOT NULL,
    popularity_score INTEGER NOT NULL,
    insight TEXT NOT NULL,
    trend_direction VARCHAR(20) DEFAULT 'stable',
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category, dish, source)
);
