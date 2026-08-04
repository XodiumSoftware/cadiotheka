CREATE TABLE IF NOT EXISTS platforms (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    color TEXT NOT NULL
);

INSERT OR IGNORE INTO platforms (id, label, color) VALUES
    ('blender', 'Blender', 'color:#c2410c'),
    ('freecad', 'FreeCAD', 'color:#1d4ed8'),
    ('sketchup', 'SketchUp', 'color:#b91c1c'),
    ('fusion_360', 'Fusion 360', 'color:#a16207'),
    ('kicad', 'KiCad', 'color:#15803d'),
    ('autocad', 'AutoCAD', 'color:#7f1d1d'),
    ('solidworks', 'SolidWorks', 'color:#991b1b'),
    ('onshape', 'Onshape', 'color:#374151'),
    ('tinkercad', 'Tinkercad', 'color:#0e7490'),
    ('step', 'STEP', 'color:#4b5563'),
    ('mesh', 'Mesh', 'color:#4b5563');

-- Migrate any legacy Tailwind class strings to inline CSS color values.
UPDATE platforms SET color = 'color:#c2410c' WHERE id = 'blender';
UPDATE platforms SET color = 'color:#1d4ed8' WHERE id = 'freecad';
UPDATE platforms SET color = 'color:#b91c1c' WHERE id = 'sketchup';
UPDATE platforms SET color = 'color:#a16207' WHERE id = 'fusion_360';
UPDATE platforms SET color = 'color:#15803d' WHERE id = 'kicad';
UPDATE platforms SET color = 'color:#7f1d1d' WHERE id = 'autocad';
UPDATE platforms SET color = 'color:#991b1b' WHERE id = 'solidworks';
UPDATE platforms SET color = 'color:#374151' WHERE id = 'onshape';
UPDATE platforms SET color = 'color:#0e7490' WHERE id = 'tinkercad';
UPDATE platforms SET color = 'color:#4b5563' WHERE id = 'step';
UPDATE platforms SET color = 'color:#4b5563' WHERE id = 'mesh';
