-- Add Salt to Users
ALTER TABLE users
    ADD COLUMN salt text NOT NULL;

