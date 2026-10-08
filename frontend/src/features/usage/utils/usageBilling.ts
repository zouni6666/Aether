import type { UsageRecord } from '@/api/usageRecords'

export type UsageBillingSnapshot = Pick<
  UsageRecord,
  'billing_multiplier' | 'billing_cost' | 'routing_group_id' | 'routing_group_name'
>

function nonnegativeFinite(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : undefined
}

export function resolveUsageBilling(record: UsageBillingSnapshot & { cost: number, actual_cost?: number | null }) {
  const capturedMultiplier = nonnegativeFinite(record.billing_multiplier)
  const multiplier = capturedMultiplier ?? 1

  // A present billing_cost is authoritative, including null (for example when
  // the server rejected an overflowed product). Never replace it with a guess.
  if (record.billing_cost !== undefined) {
    const capturedCost = nonnegativeFinite(record.billing_cost)
    return { multiplier, cost: capturedCost ?? null }
  }

  if (capturedMultiplier !== undefined) {
    const baseCost = nonnegativeFinite(record.cost)
    const derivedCost = baseCost === undefined ? null : baseCost * multiplier
    return { multiplier, cost: derivedCost !== null && Number.isFinite(derivedCost) ? derivedCost : null }
  }

  // Before group snapshots were introduced, actual_cost was the only captured
  // charge. Falling back to the catalogue/base cost would misstate old records.
  const historicalCost = nonnegativeFinite(record.actual_cost)
  return { multiplier, cost: historicalCost ?? null }
}

/** Request snapshots survive sparse updates; an accepted zero is a real value. */
export function mergeUsageBillingSnapshot(
  existing: UsageBillingSnapshot & { cost?: number | null },
  next: UsageBillingSnapshot & { cost?: number | null },
  acceptNext = true,
): UsageBillingSnapshot {
  const groupId = (acceptNext ? next.routing_group_id?.trim() : undefined) || existing.routing_group_id
  const sameGroup = !groupId || !existing.routing_group_id || groupId === existing.routing_group_id
  const nextMultiplier = acceptNext ? nonnegativeFinite(next.billing_multiplier) : undefined
  const existingMultiplier = sameGroup ? nonnegativeFinite(existing.billing_multiplier) : undefined
  const multiplierChanged = nextMultiplier !== undefined && nextMultiplier !== (existingMultiplier ?? 1)
  const nextBaseCost = acceptNext ? nonnegativeFinite(next.cost) : undefined
  const baseCostChanged = nextBaseCost !== undefined && nextBaseCost !== nonnegativeFinite(existing.cost)
  const nextCapturedCost = acceptNext && next.billing_cost !== undefined
    ? (nonnegativeFinite(next.billing_cost) ?? null)
    : undefined
  const existingCapturedCost = existing.billing_cost !== undefined
    ? (nonnegativeFinite(existing.billing_cost) ?? null)
    : undefined

  return {
    routing_group_id: groupId,
    routing_group_name: (acceptNext ? next.routing_group_name?.trim() : undefined)
      || (sameGroup ? existing.routing_group_name : undefined),
    billing_multiplier: nextMultiplier ?? existingMultiplier,
    billing_cost: nextCapturedCost !== undefined
      ? nextCapturedCost
      : !sameGroup && nextMultiplier === undefined
        ? null
        : sameGroup && (existingCapturedCost === null || (!multiplierChanged && !baseCostChanged))
          ? existingCapturedCost
          : undefined,
  }
}
