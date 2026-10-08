import client from '../client'

export type HealthObjectKind = 'api_format' | 'model' | 'provider'
export type PublicHealthObjectKind = Exclude<HealthObjectKind, 'provider'>
export type ServiceHealthStatus = 'healthy' | 'degraded' | 'unavailable' | 'unknown'
export type HealthWindow = '1h' | '6h' | '24h' | '72h'

export interface HealthRatio {
  numerator: number
  denominator: number
  value: number | null
}

export interface PublicHealthObject {
  id: string
  kind: HealthObjectKind
  name: string
  status: ServiceHealthStatus
  request_count: number
  request_success: HealthRatio
  service_availability: HealthRatio
  coverage: {
    status: 'complete' | 'partial'
    sample_status: 'empty' | 'insufficient' | 'sufficient'
    classified_count: number
    unknown_failure_count: number
    excluded_count: number
    exclusion_policy: string
  }
  average_latency_ms: number | null
  latency_sample_count: number
  last_request_at: string | null
  timeline: Array<{
    from: string
    to: string
    status: ServiceHealthStatus
    service_availability: HealthRatio
    unknown_failure_count: number
  }>
}

export interface AdminHealthObject extends PublicHealthObject {
  source_value: string
  attempts: {
    succeeded_count: number
    failed_count: number
    in_progress_count: number
    cancelled_count: number
    success: HealthRatio
  }
}

export interface HealthMeta {
  schema_version: 2
  metric_version: string
  scope: { kind: 'published' | 'authenticated' | 'installation'; object_kind: HealthObjectKind }
  range: { from: string; to: string; timezone: 'UTC'; time_basis: string }
  generated_at: string
  data_through: string | null
  freshness: 'current' | 'stale' | 'unknown'
  policy: { version: string; minimum_samples: number; healthy_threshold: number; degraded_threshold: number }
}

export interface HealthEnvelope<T> { meta: HealthMeta; data: T }
export interface HealthSummaryV2 {
  status: ServiceHealthStatus
  object_count: number
  healthy_count: number
  degraded_count: number
  unavailable_count: number
  unknown_count: number
  requests: PublicHealthObject
}
export interface HealthObjectsPage<T> { items: T[]; total: number; limit: number; offset: number }
export interface HealthQuery { kind: HealthObjectKind; window: HealthWindow; limit?: number; offset?: number }
export type PublicHealthQuery = Omit<HealthQuery, 'kind'> & { kind: PublicHealthObjectKind }
export interface HealthPublication {
  enabled: boolean
  objects: Array<{ public_id: string; kind: PublicHealthObjectKind; value: string; display_name: string }>
}

const adminRoot = '/api/admin/endpoints/health/v2'
const publicRoot = '/api/public/health/v2'
const userRoot = '/api/users/me/health/v2'

export async function getAdminHealthSummary(query: HealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthSummaryV2>>(`${adminRoot}/summary`, { params: query, signal })).data
}
export async function getPublicHealthSummaryV2(query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthSummaryV2>>(`${publicRoot}/summary`, { params: query, signal })).data
}
export async function getAdminHealthObjects(query: HealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthObjectsPage<AdminHealthObject>>>(`${adminRoot}/objects`, { params: query, signal })).data
}
export async function getPublicHealthObjects(query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthObjectsPage<PublicHealthObject>>>(`${publicRoot}/objects`, { params: query, signal })).data
}
export async function getAdminHealthObject(id: string, query: HealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<AdminHealthObject>>(`${adminRoot}/objects/${encodeURIComponent(id)}`, { params: query, signal })).data
}
export async function getPublicHealthObject(id: string, query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<PublicHealthObject>>(`${publicRoot}/objects/${encodeURIComponent(id)}`, { params: query, signal })).data
}
export async function getUserHealthSummary(query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthSummaryV2>>(`${userRoot}/summary`, { params: query, signal })).data
}
export async function getUserHealthObjects(query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<HealthObjectsPage<PublicHealthObject>>>(`${userRoot}/objects`, { params: query, signal })).data
}
export async function getUserHealthObject(id: string, query: PublicHealthQuery, signal?: AbortSignal) {
  return (await client.get<HealthEnvelope<PublicHealthObject>>(`${userRoot}/objects/${encodeURIComponent(id)}`, { params: query, signal })).data
}
export async function getHealthPublication() {
  return (await client.get<HealthPublication>(`${adminRoot}/publication`)).data
}
export async function saveHealthPublication(publication: HealthPublication) {
  return (await client.put<HealthPublication>(`${adminRoot}/publication`, publication)).data
}
