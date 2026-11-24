-- Drop the retired `orthographic` camera flag from every account's
-- `viewer_preferences` blob. Accounts without the key are left untouched.
UPDATE accounts
SET viewer_preferences = json_remove(viewer_preferences, '$.orthographic')
WHERE json_valid(viewer_preferences)
  AND json_type(viewer_preferences, '$.orthographic') IS NOT NULL;
