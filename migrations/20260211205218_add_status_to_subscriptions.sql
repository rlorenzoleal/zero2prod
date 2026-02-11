-- Add status to Subscription Table
ALTER TABLE subscriptions
    ADD COLUMN status text NULL;

