export interface ActivityHeatmapDay {
  date: string
  requests: number
  total_tokens?: number
  total_cost?: number
  actual_total_cost?: number
}

export interface ActivityHeatmap {
  start_date: string
  end_date: string
  total_days: number
  max_requests: number
  days: ActivityHeatmapDay[]
}
