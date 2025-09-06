-- Create jars table
-- Compatible with both PostgreSQL and SQLite
CREATE TABLE "jars" (
    "id" INTEGER PRIMARY KEY,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "minimal_donation" REAL NOT NULL,
    "total_amount" REAL NOT NULL DEFAULT 0.0,
    "total_donations" INTEGER NOT NULL DEFAULT 0,
    "alias" TEXT NOT NULL UNIQUE,
    "goal" REAL,
    "owner_id" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("owner_id") REFERENCES "users"("id")
);
