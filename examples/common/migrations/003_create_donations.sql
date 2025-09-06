-- Create donations table
-- Compatible with both PostgreSQL and SQLite  
-- Note: Using TEXT for UUID in SQLite, PostgreSQL will use native UUID type
CREATE TABLE "donations" (
    "id" TEXT PRIMARY KEY,
    "amount" REAL NOT NULL,
    "tip" REAL NOT NULL,
    "transaction_id" TEXT,
    "note" TEXT,
    "is_payed" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_refunded" BOOLEAN NOT NULL DEFAULT FALSE,
    "jar_id" INTEGER NOT NULL,
    "payer_id" INTEGER NOT NULL,
    "payed_at" TIMESTAMP,
    "refunded_at" TIMESTAMP,
    "deleted_at" TIMESTAMP,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("jar_id") REFERENCES "jars"("id"),
    FOREIGN KEY ("payer_id") REFERENCES "users"("id")
);
