CREATE TABLE properties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    price REAL NOT NULL,
    description TEXT,
    location TEXT NOT NULL,
    bedrooms INTEGER,
    bathrooms INTEGER,
    square_feet REAL,
    property_type TEXT NOT NULL CHECK (property_type IN ('house', 'apartment', 'land', 'commercial')),
    listing_type TEXT NOT NULL CHECK (listing_type IN ('sale', 'rent')),
    status TEXT NOT NULL CHECK (status IN ('active', 'pending', 'sold', 'rented')),
    owner_id INTEGER NOT NULL,
    agent_id INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES users (id),
    FOREIGN KEY (agent_id) REFERENCES users (id)
);