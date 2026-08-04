CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    color TEXT NOT NULL
);

INSERT OR IGNORE INTO tags (id, label, color) VALUES
    ('3d_model', '3D Model', 'background-color:#1d4ed8;color:#ffffff'),
    ('2d_drawing', '2D Drawing', 'background-color:#0e7490;color:#ffffff'),
    ('parametric', 'Parametric', 'background-color:#7e22ce;color:#ffffff'),
    ('fabrication', 'Fabrication', 'background-color:#c2410c;color:#ffffff'),
    ('robotics', 'Robotics', 'background-color:#b91c1c;color:#ffffff'),
    ('furniture', 'Furniture', 'background-color:#92400e;color:#ffffff'),
    ('vehicle', 'Vehicle', 'background-color:#15803d;color:#ffffff'),
    ('architecture', 'Architecture', 'background-color:#374151;color:#ffffff'),
    ('electronics', 'Electronics', 'background-color:#ca8a04;color:#ffffff'),
    ('tooling', 'Tooling', 'background-color:#475569;color:#ffffff'),
    ('lighting', 'Lighting', 'background-color:#d97706;color:#ffffff'),
    ('diy', 'DIY', 'background-color:#ea580c;color:#ffffff'),
    ('interior', 'Interior', 'background-color:#be123c;color:#ffffff'),
    ('engineering', 'Engineering', 'background-color:#334155;color:#ffffff'),
    ('aerospace', 'Aerospace', 'background-color:#0369a1;color:#ffffff'),
    ('decor', 'Decor', 'background-color:#e11d48;color:#ffffff'),
    ('medical', 'Medical', 'background-color:#047857;color:#ffffff'),
    ('game_asset', 'Game Asset', 'background-color:#be185d;color:#ffffff'),
    ('art', 'Art', 'background-color:#a21caf;color:#ffffff'),
    ('educational', 'Educational', 'background-color:#0f766e;color:#ffffff'),
    ('wip', 'WIP', 'background-color:#4d7c0f;color:#ffffff');

-- Migrate any legacy Tailwind class strings to inline CSS color values.
UPDATE tags SET color = 'background-color:#1d4ed8;color:#ffffff' WHERE id = '3d_model';
UPDATE tags SET color = 'background-color:#0e7490;color:#ffffff' WHERE id = '2d_drawing';
UPDATE tags SET color = 'background-color:#7e22ce;color:#ffffff' WHERE id = 'parametric';
UPDATE tags SET color = 'background-color:#c2410c;color:#ffffff' WHERE id = 'fabrication';
UPDATE tags SET color = 'background-color:#b91c1c;color:#ffffff' WHERE id = 'robotics';
UPDATE tags SET color = 'background-color:#92400e;color:#ffffff' WHERE id = 'furniture';
UPDATE tags SET color = 'background-color:#15803d;color:#ffffff' WHERE id = 'vehicle';
UPDATE tags SET color = 'background-color:#374151;color:#ffffff' WHERE id = 'architecture';
UPDATE tags SET color = 'background-color:#ca8a04;color:#ffffff' WHERE id = 'electronics';
UPDATE tags SET color = 'background-color:#475569;color:#ffffff' WHERE id = 'tooling';
UPDATE tags SET color = 'background-color:#d97706;color:#ffffff' WHERE id = 'lighting';
UPDATE tags SET color = 'background-color:#ea580c;color:#ffffff' WHERE id = 'diy';
UPDATE tags SET color = 'background-color:#be123c;color:#ffffff' WHERE id = 'interior';
UPDATE tags SET color = 'background-color:#334155;color:#ffffff' WHERE id = 'engineering';
UPDATE tags SET color = 'background-color:#0369a1;color:#ffffff' WHERE id = 'aerospace';
UPDATE tags SET color = 'background-color:#e11d48;color:#ffffff' WHERE id = 'decor';
UPDATE tags SET color = 'background-color:#047857;color:#ffffff' WHERE id = 'medical';
UPDATE tags SET color = 'background-color:#be185d;color:#ffffff' WHERE id = 'game_asset';
UPDATE tags SET color = 'background-color:#a21caf;color:#ffffff' WHERE id = 'art';
UPDATE tags SET color = 'background-color:#0f766e;color:#ffffff' WHERE id = 'educational';
UPDATE tags SET color = 'background-color:#4d7c0f;color:#ffffff' WHERE id = 'wip';
