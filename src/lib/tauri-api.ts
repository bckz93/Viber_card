import { invoke } from '@tauri-apps/api/core'
import type { PlayerStats, WeekRange } from '../types/stats'

export function getPlayerCard(): Promise<PlayerStats> {
  return invoke('get_player_card')
}

export function savePngFile(path: string, base64Data: string): Promise<void> {
  return invoke('save_png_file', { path, base64Data })
}

export function listAvailableWeeks(): Promise<WeekRange[]> {
  return invoke('list_available_weeks')
}

export function getStatsForRange(start: string, end: string): Promise<PlayerStats> {
  return invoke('get_stats_for_range', { start, end })
}
