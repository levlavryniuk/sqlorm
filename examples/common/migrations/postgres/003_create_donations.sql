CREATE TABLE "donation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "amount" DOUBLE PRECISION NOT NULL,
    "tip" DOUBLE PRECISION NOT NULL,
    "transaction_id" TEXT,
    "note" TEXT,
    "is_payed" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_refunded" BOOLEAN NOT NULL DEFAULT FALSE,
    "jar_id" BIGINT NOT NULL,
    "payer_id" BIGINT NOT NULL,
    "payed_at" TIMESTAMPTZ,
    "refunded_at" TIMESTAMPTZ,
    "deleted_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY ("jar_id") REFERENCES "jar"("id"),
    FOREIGN KEY ("payer_id") REFERENCES "user"("id")
);
