-- Tags are now hardcoded as an enum in the frontend, so the D1 seed table is
-- no longer needed. Project rows still store tag wire ids as JSON arrays.
DROP TABLE IF EXISTS tags;
