-- Manual purchasing ledger; no foreign-key cascade may erase historical expenditures.
CREATE TABLE IF NOT EXISTS public.provider_expenses (
    id text PRIMARY KEY,
    client_request_id text NOT NULL UNIQUE,
    provider_id text NOT NULL,
    provider_name text NOT NULL,
    kind text NOT NULL CHECK (kind IN ('recharge', 'subscription', 'other')),
    amount numeric(20,8) NOT NULL CHECK (amount > 0),
    currency text NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),
    paid_at timestamptz NOT NULL,
    period_start timestamptz,
    period_end timestamptz,
    note text,
    external_reference text,
    created_by text,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    voided_at timestamptz,
    voided_by text,
    CONSTRAINT provider_expenses_period_check CHECK (
        (period_start IS NULL AND period_end IS NULL) OR
        (period_start IS NOT NULL AND period_end IS NOT NULL AND period_start < period_end)
    )
);
CREATE INDEX IF NOT EXISTS ix_provider_expenses_paid_at ON public.provider_expenses(paid_at, id);
CREATE INDEX IF NOT EXISTS ix_provider_expenses_provider_paid_at ON public.provider_expenses(provider_id, paid_at);
