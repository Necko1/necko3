CREATE TABLE "api_keys" (
    "id" UUID PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "key_hash" VARCHAR(64) NOT NULL UNIQUE,
    "prefix" VARCHAR(32) NOT NULL,
    "permissions" JSONB NOT NULL,
    "is_active" BOOLEAN NOT NULL DEFAULT TRUE,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
