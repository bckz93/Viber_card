// Card theme now follows the archetype/class (not a removed rank tier),
// echoing the color language of the reference art: panic = red, night = navy,
// gold rush = amber, emo = pink/violet, sage = teal, novelist = sepia,
// detective = steel blue, balanced = soft iridescent.
export interface CardTheme {
  gradient: string
  accent: string
  shineOpacity: number
}

export const ARCHETYPE_THEME: Record<string, CardTheme> = {
  'Nocturnal Panic Coder': {
    gradient: 'linear-gradient(160deg, #6b2020 0%, #3a1010 55%, #180707 100%)',
    accent: '#e66767',
    shineOpacity: 0.14,
  },
  'Nocturnal Warrior': {
    gradient: 'linear-gradient(160deg, #16233f 0%, #0d1526 55%, #05080f 100%)',
    accent: '#7fa8e0',
    shineOpacity: 0.1,
  },
  'Token Exterminator': {
    gradient: 'linear-gradient(160deg, #6b4c14 0%, #3a2a0a 55%, #1c1404 100%)',
    accent: '#f2b83d',
    shineOpacity: 0.16,
  },
  'Spam Cannon': {
    gradient: 'linear-gradient(160deg, #35424f 0%, #1c242c 55%, #0c1014 100%)',
    accent: '#8fb3d9',
    shineOpacity: 0.12,
  },
  'Emo-Driven Coder': {
    gradient: 'linear-gradient(160deg, #5a2159 0%, #33143a 55%, #170a1c 100%)',
    accent: '#e07be0',
    shineOpacity: 0.14,
  },
  'Self-Reliant Sage': {
    gradient: 'linear-gradient(160deg, #114a3f 0%, #0a2b25 55%, #041512 100%)',
    accent: '#4fd6b0',
    shineOpacity: 0.14,
  },
  'The Novelist': {
    gradient: 'linear-gradient(160deg, #5a3a1e 0%, #33210f 55%, #170f06 100%)',
    accent: '#d9a45f',
    shineOpacity: 0.1,
  },
  'Rapid-Fire Debugger': {
    gradient: 'linear-gradient(160deg, #223a4f 0%, #13212e 55%, #080e14 100%)',
    accent: '#6fa9d8',
    shineOpacity: 0.12,
  },
  'Balanced Vibe Coder': {
    gradient: 'linear-gradient(160deg, #3a2f5a 0%, #201a33 55%, #0e0b17 100%)',
    accent: '#b39bff',
    shineOpacity: 0.18,
  },
  Unranked: {
    gradient: 'linear-gradient(160deg, #2a2d33 0%, #1a1c20 55%, #101114 100%)',
    accent: '#6b7280',
    shineOpacity: 0.04,
  },
}

export const DEFAULT_THEME: CardTheme = ARCHETYPE_THEME['Balanced Vibe Coder']

export function themeFor(archetype: string): CardTheme {
  return ARCHETYPE_THEME[archetype] ?? DEFAULT_THEME
}
