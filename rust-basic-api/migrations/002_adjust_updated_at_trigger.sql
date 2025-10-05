-- Update trigger function to use clock_timestamp for higher precision.
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CLOCK_TIMESTAMP();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
