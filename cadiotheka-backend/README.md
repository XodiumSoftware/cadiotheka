# Cadiotheka Backend

Cloudflare Pages Functions backend for Cadiotheka, powered by `workers-rs` and D1.

## Setup

1. Create a D1 database:
   ```bash
   npx wrangler d1 create cadiotheka-db
   ```

2. Update `wrangler.toml` with the database ID from step 1.

3. Apply the schema:
   ```bash
   npx wrangler d1 execute cadiotheka-db --file=schema.sql
   ```

4. Build and deploy:
   ```bash
   npx wrangler deploy
   ```

## Local Development

```bash
npx wrangler dev
```
