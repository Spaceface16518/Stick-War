# Sandbox behavior and animation pass

The three-archer attack/defend reproduction failed for both team assignments before this pass: the defending army never fired. The old mine-centered target filter also excluded attackers beside the home statue. Regression scenarios now exercise both directions on the actual Rapier simulation.

## Order behavior

- **Defend:** hold the home formation when quiet. Share awareness of intruders throughout the home corridor, including behind the statue. Respond to ranged threats against friendly troops, miners, or the statue, even outside an individual recruit's activation radius. Intercept within the team's half of the arena; never chase through midfield or target the opposing statue. Return to formation after the threat withdraws.
- **Attack:** advance, engage opposing combat units before structures, and react to close melee pressure rather than remaining locked onto a distant target.
- **Archers:** defending archers shoot from their position when in range. Backpedal against swordsmen while facing the target; archers do not retreat simply because another archer is in range.
- **Retreat:** cancel autonomous windups immediately, retain cooldowns and carried gold, and withdraw around the statue. A possessed unit continues to obey its player.
- **Reinforcements:** muster behind the statue at X = ±30.4 m. Outer entry lanes provide a clear forward path. Overflow fills vacant slots toward the front, rejecting statue footprints and occupied positions. This also makes the initial miner journey longer, symmetrically for both teams; costs, combat statistics, and gathering amounts are unchanged.

## Scenarios checked

| Scenario | Required observation |
| --- | --- |
| 3 archers attack vs 3 defend, both team assignments | Both armies fire; defenders are not passive targets. |
| 3 swordsmen already attacking a statue; add 3 defenders | Recruits enter from the rear and damage the besiegers. |
| Existing defenders ahead of a siege, outside personal activation radius | Turn backward and intercept the threat to their statue. |
| Sword/sword, sword/archer, archer/sword, archer/archer | Contact produces combat or active interception, not an inert defending army. |
| Both armies defend | Stay home; no attacks on remote units or statues. |
| Fleeing target beyond midfield | Drop pursuit and return to formation. |
| Archer already targeting a distant archer; a sword approaches | Recognize the closer melee threat. |
| Mixed armies attack, retreat, then defend | No new autonomous attacks during withdrawal; regroup without stuck columns. |
| 100 units placed while paused | No overlapping spawn slots or placement inside a statue. |
| Desktop and landscape touch | Both possessed classes, camera/input transitions, held attacks, restart/menu, and the three-archer reproduction. |

`tests/behavior.test.ts` contains complete sandbox scenarios; `tests/combat.test.ts` covers targeted edge cases. Browser tests exercise the real DOM controls and shipped GLBs. All scenarios use default combat values, except the existing explicitly labeled performance endurance fixture.

## Animation corrections

The previous bow grip was offset from the palm, and local arm rotations tipped the bow across the torso. Weapon geometry now binds to named palm sockets. Blender bakes two-bone arm posing with independent wrist orientation, smooth anticipation/contact/recovery, torso follow-through, and improved locomotion. The pick uses both hands. The bow has a deforming string, nocked arrow, release, and reload motion. First-person arms have their own rig and attack clips.

A presentation-only animation controller retimes the 35% contact/release marker to the captured simulation windup. Recovery uses the same attack's original cooldown, so hot reloads and possession cannot change a swing halfway through. Attack animations continue coherently through brief hit reactions; they no longer restart from repeated phase changes. Turning is smoothed, and backpedaling reverses the walk cycle. A separate leg animation layer keeps locomotion playing underneath attack recovery, avoiding sliding feet. Headless presentation tests verify contact timing and that pause freezes both layers.

`evidence/animation-poses.png` shows exported GLB samples at 0%, 21%, 35%, 62%, and 90% of each clip (mining contact is at 62%; combat release is at 35%). Browser assertions verify drawn-string direction, arrow height, and palm/grip alignment in close, commander, and first-person archer assets. Asset validation also requires first-person clips and joints.

Physical-phone/Safari testing remains outside this local Chrome and touch-emulation pass.
