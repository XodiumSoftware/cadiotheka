-- Add account-scoped viewer preferences so users can persist settings
-- (such as the unlocked view gizmo position) across all 3D views.
ALTER TABLE accounts ADD COLUMN viewer_preferences TEXT NOT NULL DEFAULT '{}';
