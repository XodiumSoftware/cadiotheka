CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    color TEXT NOT NULL
);

INSERT OR IGNORE INTO tags (id, label, color) VALUES
    ('3d_model', '3D Model', 'bg-blue-700 text-white'),
    ('2d_drawing', '2D Drawing', 'bg-cyan-700 text-white'),
    ('parametric', 'Parametric', 'bg-purple-700 text-white'),
    ('fabrication', 'Fabrication', 'bg-orange-700 text-white'),
    ('robotics', 'Robotics', 'bg-red-700 text-white'),
    ('furniture', 'Furniture', 'bg-amber-800 text-white'),
    ('vehicle', 'Vehicle', 'bg-green-700 text-white'),
    ('architecture', 'Architecture', 'bg-gray-700 text-white'),
    ('electronics', 'Electronics', 'bg-yellow-600 text-white'),
    ('tooling', 'Tooling', 'bg-slate-600 text-white'),
    ('lighting', 'Lighting', 'bg-amber-600 text-white'),
    ('diy', 'DIY', 'bg-orange-600 text-white'),
    ('interior', 'Interior', 'bg-rose-700 text-white'),
    ('engineering', 'Engineering', 'bg-slate-700 text-white'),
    ('aerospace', 'Aerospace', 'bg-sky-700 text-white'),
    ('decor', 'Decor', 'bg-rose-600 text-white'),
    ('medical', 'Medical', 'bg-emerald-700 text-white'),
    ('game_asset', 'Game Asset', 'bg-pink-700 text-white'),
    ('art', 'Art', 'bg-fuchsia-700 text-white'),
    ('educational', 'Educational', 'bg-teal-700 text-white'),
    ('wip', 'WIP', 'bg-lime-700 text-white');
