import json

with open('cadiotheka-frontend/test_data/cards.json', 'r', encoding='utf-8') as f:
    data = json.load(f)

cards = data['cards']
rows = []
for c in cards:
    rows.append((
        c['id'],
        c['title'].replace("'", "''"),
        c['author'].replace("'", "''"),
        c['author_id'],
        c['description'].replace("'", "''"),
        c['extended_desc'].replace("'", "''"),
        json.dumps(c['tags']),
        json.dumps(c['supported_platforms']),
        c['downloads'],
        c['favorites'],
        c['timestamp'],
        'NULL' if c['icon_url'] is None else "'" + c['icon_url'].replace("'", "''") + "'"
    ))

lines = ['INSERT INTO projects (id, title, author, author_id, description, extended_desc, tags, supported_platforms, downloads, favorites, timestamp, icon_url) VALUES']
for i, r in enumerate(rows):
    terminator = ');' if i == len(rows) - 1 else ','
    lines.append(f"('{r[0]}', '{r[1]}', '{r[2]}', '{r[3]}', '{r[4]}', '{r[5]}', '{r[6]}', '{r[7]}', {r[8]}, {r[9]}, '{r[10]}', {r[11]}){terminator}")

with open('cadiotheka-backend/scripts/seed_projects.sql', 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines))
    f.write('\n')

print(f"Wrote {len(rows)} project rows.")
