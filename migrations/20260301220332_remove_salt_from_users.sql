-- Remove Salt from Users
ALTER TABLE users
    DROP COLUMN salt;

