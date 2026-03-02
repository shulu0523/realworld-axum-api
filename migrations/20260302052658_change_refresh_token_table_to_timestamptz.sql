-- Add migration script here
-- Modify refresh_tokens table to use TIMESTAMPTZ instead of TIMESTAMP
ALTER TABLE refresh_tokens
    ALTER COLUMN expires_at TYPE TIMESTAMP WITH TIME ZONE,
    ALTER COLUMN created_at TYPE TIMESTAMP WITH TIME ZONE,
    ALTER COLUMN used_at TYPE TIMESTAMP WITH TIME ZONE;
