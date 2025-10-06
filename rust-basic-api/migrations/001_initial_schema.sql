-- Users table storing core identity and contact details
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index supporting fast lookup by unique email address
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);

-- Index supporting queries ordered by creation time
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users (created_at DESC);

-- Trigger function ensuring updated_at reflects the most recent modification
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger wiring the updated_at function to run before any row update
CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
