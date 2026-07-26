-- PostgreSQL and legacy SQLite exports encode timestamps as ISO/SQL datetime text.
-- Normalize imported and repository-facing timestamps back to the INTEGER Unix-second contract.
-- Numeric strings already receive INTEGER affinity on insert; unparseable text is preserved.

UPDATE "users"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "users"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "users"
SET "last_login_at" = CAST(strftime('%s', trim("last_login_at")) AS INTEGER)
WHERE typeof("last_login_at") = 'text'
  AND strftime('%s', trim("last_login_at")) IS NOT NULL;

UPDATE "users"
SET "privacy_policy_accepted_at" = CAST(strftime('%s', trim("privacy_policy_accepted_at")) AS INTEGER)
WHERE typeof("privacy_policy_accepted_at") = 'text'
  AND strftime('%s', trim("privacy_policy_accepted_at")) IS NOT NULL;

UPDATE "api_keys"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "api_keys"
SET "last_used_at" = CAST(strftime('%s', trim("last_used_at")) AS INTEGER)
WHERE typeof("last_used_at") = 'text'
  AND strftime('%s', trim("last_used_at")) IS NOT NULL;

UPDATE "api_keys"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "api_keys"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "providers"
SET "quota_last_reset_at" = CAST(strftime('%s', trim("quota_last_reset_at")) AS INTEGER)
WHERE typeof("quota_last_reset_at") = 'text'
  AND strftime('%s', trim("quota_last_reset_at")) IS NOT NULL;

UPDATE "providers"
SET "quota_expires_at" = CAST(strftime('%s', trim("quota_expires_at")) AS INTEGER)
WHERE typeof("quota_expires_at") = 'text'
  AND strftime('%s', trim("quota_expires_at")) IS NOT NULL;

UPDATE "providers"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "providers"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "last_429_at" = CAST(strftime('%s', trim("last_429_at")) AS INTEGER)
WHERE typeof("last_429_at") = 'text'
  AND strftime('%s', trim("last_429_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "last_probe_increase_at" = CAST(strftime('%s', trim("last_probe_increase_at")) AS INTEGER)
WHERE typeof("last_probe_increase_at") = 'text'
  AND strftime('%s', trim("last_probe_increase_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "last_used_at" = CAST(strftime('%s', trim("last_used_at")) AS INTEGER)
WHERE typeof("last_used_at") = 'text'
  AND strftime('%s', trim("last_used_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "last_models_fetch_at" = CAST(strftime('%s', trim("last_models_fetch_at")) AS INTEGER)
WHERE typeof("last_models_fetch_at") = 'text'
  AND strftime('%s', trim("last_models_fetch_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "oauth_invalid_at" = CAST(strftime('%s', trim("oauth_invalid_at")) AS INTEGER)
WHERE typeof("oauth_invalid_at") = 'text'
  AND strftime('%s', trim("oauth_invalid_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "provider_api_keys"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "provider_endpoints"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "provider_endpoints"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "global_models"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "global_models"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "models"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "models"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "auth_modules"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "auth_modules"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "oauth_providers"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "oauth_providers"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "user_oauth_links"
SET "linked_at" = CAST(strftime('%s', trim("linked_at")) AS INTEGER)
WHERE typeof("linked_at") = 'text'
  AND strftime('%s', trim("linked_at")) IS NOT NULL;

UPDATE "user_oauth_links"
SET "last_login_at" = CAST(strftime('%s', trim("last_login_at")) AS INTEGER)
WHERE typeof("last_login_at") = 'text'
  AND strftime('%s', trim("last_login_at")) IS NOT NULL;

UPDATE "user_groups"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "user_groups"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "user_group_members"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "proxy_nodes"
SET "last_heartbeat_at" = CAST(strftime('%s', trim("last_heartbeat_at")) AS INTEGER)
WHERE typeof("last_heartbeat_at") = 'text'
  AND strftime('%s', trim("last_heartbeat_at")) IS NOT NULL;

UPDATE "proxy_nodes"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "proxy_nodes"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "proxy_nodes"
SET "tunnel_connected_at" = CAST(strftime('%s', trim("tunnel_connected_at")) AS INTEGER)
WHERE typeof("tunnel_connected_at") = 'text'
  AND strftime('%s', trim("tunnel_connected_at")) IS NOT NULL;

UPDATE "system_configs"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "system_configs"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "wallets"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "wallets"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "wallet_transactions"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "wallet_daily_usage_ledgers"
SET "first_finalized_at" = CAST(strftime('%s', trim("first_finalized_at")) AS INTEGER)
WHERE typeof("first_finalized_at") = 'text'
  AND strftime('%s', trim("first_finalized_at")) IS NOT NULL;

UPDATE "wallet_daily_usage_ledgers"
SET "last_finalized_at" = CAST(strftime('%s', trim("last_finalized_at")) AS INTEGER)
WHERE typeof("last_finalized_at") = 'text'
  AND strftime('%s', trim("last_finalized_at")) IS NOT NULL;

UPDATE "wallet_daily_usage_ledgers"
SET "aggregated_at" = CAST(strftime('%s', trim("aggregated_at")) AS INTEGER)
WHERE typeof("aggregated_at") = 'text'
  AND strftime('%s', trim("aggregated_at")) IS NOT NULL;

UPDATE "wallet_daily_usage_ledgers"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "wallet_daily_usage_ledgers"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "payment_orders"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "payment_orders"
SET "paid_at" = CAST(strftime('%s', trim("paid_at")) AS INTEGER)
WHERE typeof("paid_at") = 'text'
  AND strftime('%s', trim("paid_at")) IS NOT NULL;

UPDATE "payment_orders"
SET "credited_at" = CAST(strftime('%s', trim("credited_at")) AS INTEGER)
WHERE typeof("credited_at") = 'text'
  AND strftime('%s', trim("credited_at")) IS NOT NULL;

UPDATE "payment_orders"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "payment_callbacks"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "payment_callbacks"
SET "processed_at" = CAST(strftime('%s', trim("processed_at")) AS INTEGER)
WHERE typeof("processed_at") = 'text'
  AND strftime('%s', trim("processed_at")) IS NOT NULL;

UPDATE "refund_requests"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "refund_requests"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "refund_requests"
SET "processed_at" = CAST(strftime('%s', trim("processed_at")) AS INTEGER)
WHERE typeof("processed_at") = 'text'
  AND strftime('%s', trim("processed_at")) IS NOT NULL;

UPDATE "refund_requests"
SET "completed_at" = CAST(strftime('%s', trim("completed_at")) AS INTEGER)
WHERE typeof("completed_at") = 'text'
  AND strftime('%s', trim("completed_at")) IS NOT NULL;

UPDATE "redeem_code_batches"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "redeem_code_batches"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "redeem_code_batches"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "redeem_codes"
SET "redeemed_at" = CAST(strftime('%s', trim("redeemed_at")) AS INTEGER)
WHERE typeof("redeemed_at") = 'text'
  AND strftime('%s', trim("redeemed_at")) IS NOT NULL;

UPDATE "redeem_codes"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "redeem_codes"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "usage"
SET "created_at_unix_ms" = CAST(strftime('%s', trim("created_at_unix_ms")) AS INTEGER)
WHERE typeof("created_at_unix_ms") = 'text'
  AND strftime('%s', trim("created_at_unix_ms")) IS NOT NULL;

UPDATE "usage"
SET "updated_at_unix_secs" = CAST(strftime('%s', trim("updated_at_unix_secs")) AS INTEGER)
WHERE typeof("updated_at_unix_secs") = 'text'
  AND strftime('%s', trim("updated_at_unix_secs")) IS NOT NULL;

UPDATE "usage"
SET "finalized_at" = CAST(strftime('%s', trim("finalized_at")) AS INTEGER)
WHERE typeof("finalized_at") = 'text'
  AND strftime('%s', trim("finalized_at")) IS NOT NULL;

UPDATE "billing_rules"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "billing_rules"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "dimension_collectors"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "dimension_collectors"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "usage_settlement_snapshots"
SET "finalized_at" = CAST(strftime('%s', trim("finalized_at")) AS INTEGER)
WHERE typeof("finalized_at") = 'text'
  AND strftime('%s', trim("finalized_at")) IS NOT NULL;

UPDATE "usage_settlement_snapshots"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "usage_settlement_snapshots"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "announcements"
SET "start_time" = CAST(strftime('%s', trim("start_time")) AS INTEGER)
WHERE typeof("start_time") = 'text'
  AND strftime('%s', trim("start_time")) IS NOT NULL;

UPDATE "announcements"
SET "end_time" = CAST(strftime('%s', trim("end_time")) AS INTEGER)
WHERE typeof("end_time") = 'text'
  AND strftime('%s', trim("end_time")) IS NOT NULL;

UPDATE "announcements"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "announcements"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "management_tokens"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "management_tokens"
SET "last_used_at" = CAST(strftime('%s', trim("last_used_at")) AS INTEGER)
WHERE typeof("last_used_at") = 'text'
  AND strftime('%s', trim("last_used_at")) IS NOT NULL;

UPDATE "management_tokens"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "management_tokens"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "video_tasks"
SET "next_poll_at" = CAST(strftime('%s', trim("next_poll_at")) AS INTEGER)
WHERE typeof("next_poll_at") = 'text'
  AND strftime('%s', trim("next_poll_at")) IS NOT NULL;

UPDATE "video_tasks"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "video_tasks"
SET "submitted_at" = CAST(strftime('%s', trim("submitted_at")) AS INTEGER)
WHERE typeof("submitted_at") = 'text'
  AND strftime('%s', trim("submitted_at")) IS NOT NULL;

UPDATE "video_tasks"
SET "completed_at" = CAST(strftime('%s', trim("completed_at")) AS INTEGER)
WHERE typeof("completed_at") = 'text'
  AND strftime('%s', trim("completed_at")) IS NOT NULL;

UPDATE "video_tasks"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "payment_gateway_configs"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "payment_gateway_configs"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "billing_plans"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "billing_plans"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

UPDATE "user_plan_entitlements"
SET "starts_at" = CAST(strftime('%s', trim("starts_at")) AS INTEGER)
WHERE typeof("starts_at") = 'text'
  AND strftime('%s', trim("starts_at")) IS NOT NULL;

UPDATE "user_plan_entitlements"
SET "expires_at" = CAST(strftime('%s', trim("expires_at")) AS INTEGER)
WHERE typeof("expires_at") = 'text'
  AND strftime('%s', trim("expires_at")) IS NOT NULL;

UPDATE "user_plan_entitlements"
SET "created_at" = CAST(strftime('%s', trim("created_at")) AS INTEGER)
WHERE typeof("created_at") = 'text'
  AND strftime('%s', trim("created_at")) IS NOT NULL;

UPDATE "user_plan_entitlements"
SET "updated_at" = CAST(strftime('%s', trim("updated_at")) AS INTEGER)
WHERE typeof("updated_at") = 'text'
  AND strftime('%s', trim("updated_at")) IS NOT NULL;

-- Do not mark the migration successful while an incompatible timestamp can still reach SQLx.
DROP TABLE IF EXISTS temp._aether_timestamp_storage_guard;
CREATE TEMP TABLE _aether_timestamp_storage_guard (
    invalid_count INTEGER NOT NULL,
    CONSTRAINT imported_timestamp_storage_must_be_integer CHECK (invalid_count = 0)
);

INSERT INTO _aether_timestamp_storage_guard (invalid_count)
SELECT COUNT(*)
FROM (
    SELECT 1 FROM "users"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "users"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "users"
    WHERE "last_login_at" IS NOT NULL AND typeof("last_login_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "users"
    WHERE "privacy_policy_accepted_at" IS NOT NULL AND typeof("privacy_policy_accepted_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "api_keys"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "api_keys"
    WHERE "last_used_at" IS NOT NULL AND typeof("last_used_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "api_keys"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "api_keys"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "providers"
    WHERE "quota_last_reset_at" IS NOT NULL AND typeof("quota_last_reset_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "providers"
    WHERE "quota_expires_at" IS NOT NULL AND typeof("quota_expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "providers"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "providers"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "last_429_at" IS NOT NULL AND typeof("last_429_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "last_probe_increase_at" IS NOT NULL AND typeof("last_probe_increase_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "last_used_at" IS NOT NULL AND typeof("last_used_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "last_models_fetch_at" IS NOT NULL AND typeof("last_models_fetch_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "oauth_invalid_at" IS NOT NULL AND typeof("oauth_invalid_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_api_keys"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_endpoints"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "provider_endpoints"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "global_models"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "global_models"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "models"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "models"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "auth_modules"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "auth_modules"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "oauth_providers"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "oauth_providers"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_oauth_links"
    WHERE "linked_at" IS NOT NULL AND typeof("linked_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_oauth_links"
    WHERE "last_login_at" IS NOT NULL AND typeof("last_login_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_groups"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_groups"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_group_members"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "proxy_nodes"
    WHERE "last_heartbeat_at" IS NOT NULL AND typeof("last_heartbeat_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "proxy_nodes"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "proxy_nodes"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "proxy_nodes"
    WHERE "tunnel_connected_at" IS NOT NULL AND typeof("tunnel_connected_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "system_configs"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "system_configs"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallets"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallets"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_transactions"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_daily_usage_ledgers"
    WHERE "first_finalized_at" IS NOT NULL AND typeof("first_finalized_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_daily_usage_ledgers"
    WHERE "last_finalized_at" IS NOT NULL AND typeof("last_finalized_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_daily_usage_ledgers"
    WHERE "aggregated_at" IS NOT NULL AND typeof("aggregated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_daily_usage_ledgers"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "wallet_daily_usage_ledgers"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_orders"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_orders"
    WHERE "paid_at" IS NOT NULL AND typeof("paid_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_orders"
    WHERE "credited_at" IS NOT NULL AND typeof("credited_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_orders"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_callbacks"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_callbacks"
    WHERE "processed_at" IS NOT NULL AND typeof("processed_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "refund_requests"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "refund_requests"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "refund_requests"
    WHERE "processed_at" IS NOT NULL AND typeof("processed_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "refund_requests"
    WHERE "completed_at" IS NOT NULL AND typeof("completed_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_code_batches"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_code_batches"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_code_batches"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_codes"
    WHERE "redeemed_at" IS NOT NULL AND typeof("redeemed_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_codes"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "redeem_codes"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage"
    WHERE "created_at_unix_ms" IS NOT NULL AND typeof("created_at_unix_ms") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage"
    WHERE "updated_at_unix_secs" IS NOT NULL AND typeof("updated_at_unix_secs") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage"
    WHERE "finalized_at" IS NOT NULL AND typeof("finalized_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "billing_rules"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "billing_rules"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "dimension_collectors"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "dimension_collectors"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage_settlement_snapshots"
    WHERE "finalized_at" IS NOT NULL AND typeof("finalized_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage_settlement_snapshots"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "usage_settlement_snapshots"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "announcements"
    WHERE "start_time" IS NOT NULL AND typeof("start_time") <> 'integer'
    UNION ALL
    SELECT 1 FROM "announcements"
    WHERE "end_time" IS NOT NULL AND typeof("end_time") <> 'integer'
    UNION ALL
    SELECT 1 FROM "announcements"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "announcements"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "management_tokens"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "management_tokens"
    WHERE "last_used_at" IS NOT NULL AND typeof("last_used_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "management_tokens"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "management_tokens"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "video_tasks"
    WHERE "next_poll_at" IS NOT NULL AND typeof("next_poll_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "video_tasks"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "video_tasks"
    WHERE "submitted_at" IS NOT NULL AND typeof("submitted_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "video_tasks"
    WHERE "completed_at" IS NOT NULL AND typeof("completed_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "video_tasks"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_gateway_configs"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "payment_gateway_configs"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "billing_plans"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "billing_plans"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_plan_entitlements"
    WHERE "starts_at" IS NOT NULL AND typeof("starts_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_plan_entitlements"
    WHERE "expires_at" IS NOT NULL AND typeof("expires_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_plan_entitlements"
    WHERE "created_at" IS NOT NULL AND typeof("created_at") <> 'integer'
    UNION ALL
    SELECT 1 FROM "user_plan_entitlements"
    WHERE "updated_at" IS NOT NULL AND typeof("updated_at") <> 'integer'
) AS invalid_timestamp_values;

DROP TABLE temp._aether_timestamp_storage_guard;
