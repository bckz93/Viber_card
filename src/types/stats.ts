// Mirrors src-tauri/src/scoring.rs::PlayerStats (serde field names).
export interface StatInsight {
  key: string
  label: string
  value: number
  explanation: string
}

export interface PlayerStats {
  vol: number
  spd: number
  nct: number
  slf: number
  emo: number
  archetype: string
  punchline: string
  insights: StatInsight[]
  sample_size: number
  total_tokens: number
  range_start: string
  range_end: string
}

// Mirrors src-tauri/src/commands.rs::WeekRange.
export interface WeekRange {
  start: string
  end: string
}
