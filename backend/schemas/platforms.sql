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
