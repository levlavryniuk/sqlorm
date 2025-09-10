CREATE TABLE "donation" (
    "id" TEXT PRIMARY KEY,
    "amount" REAL NOT NULL,
    "tip" REAL NOT NULL,
    "transaction_id" TEXT,
    "note" TEXT,
    "is_payed" INTEGER NOT NULL DEFAULT 0,
    "is_refunded" INTEGER NOT NULL DEFAULT 0,
    "jar_id" INTEGER NOT NULL,
    "payer_id" INTEGER NOT NULL,
    "payed_at" DATETIME,
    "refunded_at" DATETIME,
    "deleted_at" DATETIME,
    "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("jar_id") REFERENCES "jar"("id"),
    FOREIGN KEY ("payer_id") REFERENCES "user"("id")
);
