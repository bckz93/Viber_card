import nocturnalPanicCoder from '../../assets/archetypes/nocturnal-panic-coder.png'
import nocturnalWarrior from '../../assets/archetypes/nocturnal-warrior.png'
import tokenExterminator from '../../assets/archetypes/token-exterminator.png'
import spamCannon from '../../assets/archetypes/spam-cannon.png'
import emoDrivenCoder from '../../assets/archetypes/emo-driven-coder.png'
import selfReliantSage from '../../assets/archetypes/self-reliant-sage.png'
import theNovelist from '../../assets/archetypes/the-novelist.png'
import rapidFireDebugger from '../../assets/archetypes/rapid-fire-debugger.png'
import balancedVibeCoder from '../../assets/archetypes/balanced-vibe-coder.png'

// Keys must match the exact strings returned by src-tauri/src/scoring.rs::archetype_for.
export const ARCHETYPE_ART: Record<string, string> = {
  'Nocturnal Panic Coder': nocturnalPanicCoder,
  'Nocturnal Warrior': nocturnalWarrior,
  'Token Exterminator': tokenExterminator,
  'Spam Cannon': spamCannon,
  'Emo-Driven Coder': emoDrivenCoder,
  'Self-Reliant Sage': selfReliantSage,
  'The Novelist': theNovelist,
  'Rapid-Fire Debugger': rapidFireDebugger,
  'Balanced Vibe Coder': balancedVibeCoder,
}

export function getArchetypeArt(archetype: string): string | undefined {
  return ARCHETYPE_ART[archetype]
}
