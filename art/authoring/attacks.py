"""Attack choreography, normalized around each clip's contact/release marker."""
import math
from mathutils import Vector
from .posing import curve


def sword_pose(name, t, contact, left, right, angles):
    """Preparation, strike, follow-through, then return to a reusable guard."""
    profiles = {
        'melee_slash': {
            'grips': [(-.40, .015, 1.42), (-.13, -.53, 1.14), (.20, -.31, .95)],
            'angles': [(-.4, -.15, -.55), (1.37, -.15, -.40), (1.5, .6, .45)],
            'twist': [-.27, .20, .32],
        },
        'melee_backhand': {
            'grips': [(.14, -.23, 1.24), (-.05, -.55, 1.22), (-.44, -.15, 1.09)],
            'angles': [(.4, .85, .7), (1.32, -.25, -.6), (.55, -.75, -.6)],
            'twist': [.29, -.17, -.3],
        },
        'melee_overhead': {
            'grips': [(-.19, -.02, 1.66), (-.13, -.51, 1.13), (-.04, -.45, .85)],
            'angles': [(-.8, -.15, -.2), (1.42, -.08, -.12), (1.8, -.06, .0)],
            'twist': [-.10, .11, .16],
        },
    }
    p = profiles[name]
    prep, follow = contact*.70, contact+(1-contact)*.26
    right = curve(t, [(0, right), (.10, right), (prep, p['grips'][0]),
                      (contact, p['grips'][1]), (follow, p['grips'][2]),
                      (.90, right), (1, right)])
    angles = curve(t, [(0, angles), (prep, p['angles'][0]),
                       (contact, p['angles'][1]), (follow, p['angles'][2]),
                       (.90, angles), (1, angles)])
    torso = curve(t, [(0, (0, 0, 0)), (prep, (-.04, 0, p['twist'][0])),
                      (contact, (.14, 0, p['twist'][1])),
                      (follow, (.22 if name.endswith('overhead') else .07, 0, p['twist'][2])),
                      (.92, (0, 0, 0)), (1, (0, 0, 0))])
    left = curve(t, [(0, left), (prep, (.29, -.18, 1.16)),
                     (contact, (.36, -.10, 1.02)), (.9, left), (1, left)])
    weight = math.sin(math.pi*t)**2
    return left, right, angles, torso, weight


def bow_pose(name, t, contact, left, right):
    held = name == 'bow_hold'
    quick = name == 'bow_quick'
    # The deliberate shot raises from low guard and settles into a longer hold;
    # the quick shot draws immediately; the normal shot has a smooth lift.
    raise_at = contact*(.22 if quick else .37)
    drawn_at = contact*(.60 if held else .82)
    release = contact+(1-contact)*.12
    lower = contact+(1-contact)*.56
    grip = (.20 if held else .22, -.50, 1.42 if not quick else 1.38)
    anchor = (grip[0], -.045 if held else -.065, grip[2])
    left = curve(t, [(0, left), (raise_at, grip), (contact, grip),
                     (release, (grip[0], -.48, grip[2]-.012)), (lower, (.32, -.23, 1.09)), (1, left)])
    right = curve(t, [(0, right), (raise_at*.75, (grip[0], -.34, grip[2]-.02)),
                      (drawn_at, anchor), (contact, anchor),
                      (release, (.10, .055, grip[2]+.035)),
                      (contact+(1-contact)*.42, (-.12, .12, 1.47)),
                      (contact+(1-contact)*.72, (.18, -.16, 1.08)), (1, right)])
    # Draw hand and arrow stay together; release introduces a brief damped return.
    draw = min(1, t/max(.001, raise_at*.7)) if t <= contact else 0
    if contact < t < release:
        f = (t-contact)/(release-contact)
        draw = max(0, .13*math.exp(-f*5)*math.cos(f*math.tau*2))
    visible = .03 <= t <= contact
    torso = (.035, 0, (.045 if quick else .095)*math.sin(math.pi*t))
    return left, right, draw, visible, torso


def fp_point(x, y, z):
    return Vector((x, -z, y))


def first_person_pose(name, t, contact, grips):
    targets = {side: grip.copy() for side, grip in grips.items()}
    angles = {'l': Vector((0, 0, 0)), 'r': Vector((0, 0, 0))}
    draw, visible = 0, False
    if name.startswith('melee_'):
        profiles = {
            'melee_slash': [( .40, -.07, -.34), (-.02, -.18, -.65), (-.26, -.32, -.48)],
            'melee_backhand': [(-.22, -.14, -.38), (.06, -.20, -.68), (.43, -.24, -.43)],
            'melee_overhead': [( .19, .19, -.34), (.08, -.15, -.70), (.08, -.43, -.58)],
        }
        p = [fp_point(*v) for v in profiles[name]]
        prep, follow = contact*.70, contact+(1-contact)*.27
        targets['r'] = curve(t, [(0, grips['r']), (prep, p[0]), (contact, p[1]), (follow, p[2]), (.91, grips['r']), (1, grips['r'])])
        side = -1 if name == 'melee_backhand' else 1
        angles['r'] = curve(t, [(0, (0, 0, 0)), (prep, (.45, .18*side, -.3*side)),
                                (contact, (-1.35, -.15*side, .25*side)),
                                (follow, (-1.7, -.45*side, .45*side)), (.91, (0, 0, 0)), (1, (0, 0, 0))])
        targets['l'] += fp_point(-.025*side, -.02, -.04)*math.sin(math.pi*t)
    elif name.startswith('bow_'):
        held = name == 'bow_hold'
        raise_at = contact*(.22 if name == 'bow_quick' else .37)
        drawn_at = contact*(.60 if held else .82)
        release = contact+(1-contact)*.12
        left = fp_point(-.22, -.16 if held else -.18, -.63)
        right = fp_point(-.22, -.16 if held else -.18, -.15 if held else -.19)
        targets['l'] = curve(t, [(0, grips['l']), (raise_at, left), (contact, left),
                                 (release, left+fp_point(0, -.012, .03)), (1, grips['l'])])
        targets['r'] = curve(t, [(0, grips['r']), (raise_at*.75, left+fp_point(0, 0, .17)),
                                 (drawn_at, right), (contact, right),
                                 (release, right+fp_point(.055, .02, .07)),
                                 (contact+(1-contact)*.43, fp_point(.29, -.23, -.19)),
                                 (contact+(1-contact)*.77, fp_point(-.18, -.25, -.40)), (1, grips['r'])])
        draw = min(1, t/max(.001, raise_at*.7)) if t <= contact else 0
        visible = .025 <= t <= contact
    else:
        for side in targets:
            targets[side] += fp_point(0, .004*math.sin(t*math.tau), .002*math.cos(t*math.tau))
    return targets, angles, draw, visible
