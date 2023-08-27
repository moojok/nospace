# Running Migrator CLI

- Generate a new migration file
  ```bash
  cargo run -- migrate generate MIGRATION_NAME
  ```
- Apply all pending migrations
  ```bash
  cargo run
  ```
  ```bash
  cargo run -- up
  ```
- Apply first 10 pending migrations
  ```bash
  cargo run -- up -n 10
  ```
- Rollback last applied migrations
  ```bash
  cargo run -- down
  ```
- Rollback last 10 applied migrations
  ```bash
  cargo run -- down -n 10
  ```
- Drop all tables from the database, then reapply all migrations
  ```bash
  cargo run -- fresh
  ```
- Rollback all applied migrations, then reapply all migrations
  ```bash
  cargo run -- refresh
  ```
- Rollback all applied migrations
  ```bash
  cargo run -- reset
  ```
- Check the status of all migrations
  ```bash
  cargo run -- status
  ```
