CREATE TABLE "jar" (
    "id" BIGSERIAL PRIMARY KEY,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "minimal_donation" DOUBLE PRECISION NOT NULL,
    "total_amount" DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    "total_donations" INTEGER NOT NULL DEFAULT 0,
    "alias" TEXT NOT NULL UNIQUE,
    "hide_earnings" BOOLEAN NOT NULL DEFAULT FALSE,
    "goal" DOUBLE PRECISION,
    "owner_id" BIGINT NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("owner_id") REFERENCES "user"("id")
);
