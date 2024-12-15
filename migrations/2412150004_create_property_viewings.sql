CREATE TABLE property_viewings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    property_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    viewing_date TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('requested', 'confirmed', 'completed', 'cancelled')),
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (property_id) REFERENCES properties (id),
    FOREIGN KEY (user_id) REFERENCES users (id)
);