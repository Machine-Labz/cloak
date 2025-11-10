-- Create a sequence for atomic leaf index allocation
-- This prevents race conditions when multiple deposits arrive concurrently

-- Create the sequence, starting from 0
-- MINVALUE 0 is required because leaf indices start at 0
CREATE SEQUENCE IF NOT EXISTS leaf_index_seq
    START WITH 0
    INCREMENT BY 1
    MINVALUE 0
    NO MAXVALUE
    CACHE 1;

-- Set the sequence to the current maximum leaf index
-- This ensures we don't reuse existing indices
DO $$
DECLARE
    max_index BIGINT;
BEGIN
    -- Get the current maximum leaf index from notes table
    SELECT COALESCE(MAX(leaf_index), -1) INTO max_index FROM notes;

    -- Set the sequence to start from max_index + 1
    PERFORM setval('leaf_index_seq', max_index + 1, false);

    RAISE NOTICE 'Leaf index sequence initialized to start at %', max_index + 1;
END $$;
