UPDATE providers
SET enabled = is_active
WHERE enabled <> is_active;

UPDATE provider_endpoints
SET enabled = is_active
WHERE enabled <> is_active;

UPDATE models
SET enabled = is_active
WHERE enabled <> is_active;
