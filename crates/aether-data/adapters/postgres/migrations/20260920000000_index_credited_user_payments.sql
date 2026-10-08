-- no-transaction
-- User finance reports use the actual credited period, not order creation time.
-- Keep the index build separate from transactional schema changes.
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_payment_orders_status_credited_user
  ON public.payment_orders USING btree (status, credited_at, user_id);
