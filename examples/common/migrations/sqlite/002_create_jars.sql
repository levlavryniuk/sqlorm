CREATE TABLE "jar" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "minimal_donation" REAL NOT NULL,
    "total_amount" REAL NOT NULL DEFAULT 0.0,
    "total_donations" INTEGER NOT NULL DEFAULT 0,
    "alias" TEXT NOT NULL UNIQUE,
    "hide_earnings" INTEGER NOT NULL DEFAULT 0,
    "goal" REAL,
    "owner_id" INTEGER NOT NULL,
    "created_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "deleted_at" DATETIME,
    FOREIGN KEY ("owner_id") REFERENCES "user"("id")
);
