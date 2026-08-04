CREATE TABLE IF NOT EXISTS platforms (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    color TEXT NOT NULL
);

INSERT OR IGNORE INTO platforms (id, label, color) VALUES
    ('blender', 'Blender', 'text-orange-700'),
    ('freecad', 'FreeCAD', 'text-blue-700'),
    ('sketchup', 'SketchUp', 'text-red-700'),
    ('fusion_360', 'Fusion 360', 'text-yellow-700'),
    ('kicad', 'KiCad', 'text-green-700'),
    ('autocad', 'AutoCAD', 'text-red-900'),
    ('solidworks', 'SolidWorks', 'text-red-800'),
    ('onshape', 'Onshape', 'text-gray-700'),
    ('tinkercad', 'Tinkercad', 'text-cyan-700'),
    ('step', 'STEP', 'text-gray-600'),
    ('mesh', 'Mesh', 'text-gray-600');
