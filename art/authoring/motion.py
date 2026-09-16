"""Rig actions only. Rebuilding these never changes model geometry or weights."""
import json
import math
import bpy
from mathutils import Vector, Euler
from .geometry import ROOT
from .posing import clear_pose, anatomical, arm, leg, orient, nock, curve
from .attacks import sword_pose, bow_pose, first_person_pose

LIBRARY = json.loads((ROOT/'config/animation.json').read_text())
SIDES = [('l', 1), ('r', -1)]


def guard(kind, t, moving=False):
    swing = math.sin(t*math.tau)*(.075 if moving else .008)
    left, right, angles = Vector((.35, -.06+swing, .84)), Vector((-.34, -.22-swing, 1.04)), Vector((.25, 0, -.1))
    if kind == 'miner':
        right, angles = Vector((-.36, -.15-swing, .87)), Vector((.18, 0, .05))
    elif kind == 'archer':
        left, right, angles = Vector((.34, -.16-swing, .95)), Vector((-.30, -.16+swing, 1.04)), Vector((0, 0, 0))
    return left, right, angles


def walk(obj, name, t):
    """Linear stance travel cancels world movement; swing lifts and rolls the foot."""
    running = name == 'run'
    stride = LIBRARY['locomotion'][name]['distance']
    stance = .40 if running else .58
    a = math.sin(t*math.tau)
    down = .10 if running else .085
    bounce = (.052 if running else .024)*(1-math.cos(t*math.tau*2))
    anatomical(obj.pose.bones['pelvis'], (0, .025*a, .035*a),
               (.018*a, 0, -down+bounce))
    for side, sign in SIDES:
        phase = (t+(0 if sign == 1 else .5)) % 1
        if phase < stance:
            u = phase/stance
            y = stride*stance*(u-.5)
            pitch = -.18*max(0, 1-u/.15)+.38*max(0, (u-.70)/.30)
            lift = 0
            toe = -.45*max(0, (u-.76)/.24)
        else:
            u = (phase-stance)/(1-stance)
            smooth = u*u*(3-2*u)
            y = stride*stance*(.5-smooth)
            lift = (.22 if running else .13)*math.sin(math.pi*u)**1.2
            pitch = .38*(1-smooth)-.18*smooth
            toe = -.45*max(0, 1-u/.25)
        x = sign*.15
        if name == 'walk_back':
            y, pitch, toe = -y, -pitch, 0
        elif name.startswith('strafe_'):
            x -= y*(1 if name == 'strafe_left' else -1)
            y, pitch, toe = 0, 0, 0
        # Heel strike and toe-off rotate about the sole instead of sinking it.
        sole_height = .125*math.cos(pitch)-min(-.23*math.sin(pitch), .10*math.sin(pitch))+.012
        leg(obj, side, (x, y, sole_height+lift), pitch, toe)
    anatomical(obj.pose.bones['spine'], (.10 if running else .045, -.025*a, -.035*a))
    anatomical(obj.pose.bones['head'], (-.035, .015*a, .022*a))


def planted(obj, weight=0):
    anatomical(obj.pose.bones['pelvis'], (0, 0, .045*weight), (0, -.035*weight, -.045-.025*weight))
    for side, sign in SIDES:
        leg(obj, side, (sign*(.15+.035*weight), -.08*weight*sign, .137))


def impact(obj, name, t):
    # Fast compression, delayed neck recoil and a damped recovery. These clips
    # are converted to additive deltas, so attacks and footsteps remain visible.
    amount = curve(t, [(0, (0,)), (.12, (1,)), (.30, (.75,)), (.58, (-.15,)), (.78, (.07,)), (1, (0,))])[0]
    head = curve(t, [(0, (0,)), (.21, (1,)), (.46, (.3,)), (.68, (-.12,)), (1, (0,))])[0]
    pitch = -.22 if name == 'hit_front' else .20 if name == 'hit_back' else -.035
    roll = .18 if name == 'hit_left' else -.18 if name == 'hit_right' else .035
    anatomical(obj.pose.bones['spine'], (pitch*amount, roll*amount, roll*.4*amount), (0, 0, -.025*max(0, amount)))
    anatomical(obj.pose.bones['head'], (-pitch*.65*head, -roll*.6*head, 0))


def death(obj, name, t):
    side = name in ('death_left', 'death_right')
    sign = -1 if name in ('death_back', 'death_right') else 1
    fall = curve(t, [(0, (0,)), (.16, (.05,)), (.37, (.48,)), (.64, (1.34,)),
                     (.77, (1.54,)), (.86, (1.48,)), (1, (1.52,))])[0]
    height = curve(t, [(0, (.805,)), (.17, (.65,)), (.42, (.37,)),
                       (.68, (.36 if side else .17,)), (.80, (.38 if side else .19,)),
                       (1, (.35 if side else .15,))])[0]
    rotation = Euler((0 if side else fall*sign, fall*sign if side else 0, .025*math.sin(math.pi*t)), 'XYZ')
    hip = Vector((sign*.12*t if side else 0, 0 if side else -sign*.14*t, height))
    offset = hip-rotation.to_quaternion() @ Vector((0, 0, .85))
    anatomical(obj.pose.bones['root'], rotation, offset)
    knees = math.sin(math.pi*min(1, t/.8))
    anatomical(obj.pose.bones['thigh_l'], (-.28*knees, .12*knees, 0))
    anatomical(obj.pose.bones['shin_l'], (.70*knees, 0, 0))
    anatomical(obj.pose.bones['thigh_r'], (-.45*knees, -.13*knees, 0))
    anatomical(obj.pose.bones['shin_r'], (1.0*knees, 0, 0))
    anatomical(obj.pose.bones['spine'], (-.07*math.sin(math.pi*t), 0, .08*math.sin(math.pi*t)))
    anatomical(obj.pose.bones['head'], (.10*math.sin(math.pi*t), 0, -.07*math.sin(math.pi*t)))
    for hand, s in SIDES:
        brace = math.sin(math.pi*min(1, t/.72))
        anatomical(obj.pose.bones['upper_arm_'+hand], (-.65*brace, s*.10*min(1, t*2), s*(.15*brace+.12*t)))
        anatomical(obj.pose.bones['forearm_'+hand], (-.25*brace, 0, 0))
    if side:
        # Relax the weapon wrist into the ground plane during a sideways fall.
        anatomical(obj.pose.bones['hand_r'], (0, 0, math.pi*.5*min(1, t/.65)))
    obj.pose.bones['arrow_socket'].scale = (.001,)*3


def ground_contact(obj):
    """Bake floor contact into the root, including the held equipment envelope."""
    bpy.context.view_layer.update()
    graph = bpy.context.evaluated_depsgraph_get()
    lowest = 0
    for mesh in bpy.context.scene.objects:
        if mesh.type == 'MESH' and any(m.type == 'ARMATURE' and m.object == obj for m in mesh.modifiers):
            evaluated = mesh.evaluated_get(graph)
            lowest = min(lowest, min((evaluated.matrix_world @ v.co).z for v in evaluated.data.vertices))
    if lowest < .006:
        root = obj.pose.bones['root']
        matrix = root.matrix.copy()
        matrix.translation.z += .006-lowest
        root.matrix = matrix
        bpy.context.view_layer.update()


def humanoid_pose(obj, kind, name, t):
    bones = obj.pose.bones
    if name.startswith('hit_'):
        impact(obj, name, t)
        return
    if name.startswith('death_'):
        death(obj, name, t)
        ground_contact(obj)
        return
    moving = name in LIBRARY['locomotion'] and name != 'runThreshold'
    if moving:
        walk(obj, name, t)
    else:
        planted(obj)
        anatomical(bones['spine'], (.012*math.sin(t*math.tau), 0, .008*math.sin(t*math.tau)))
        anatomical(bones['head'], (-.008*math.sin(t*math.tau), 0, 0))
    left, right, angles = guard(kind, t, moving)
    draw, visible = 0, False
    if name.startswith('melee_'):
        contact = LIBRARY['attacks'][name]['contact']
        left, right, angles, torso, weight = sword_pose(name, t, contact, left, right, angles)
        planted(obj, weight)
        anatomical(bones['spine'], torso)
        anatomical(bones['head'], (-torso.x*.35, 0, -torso.z*.45))
    elif name.startswith('bow_'):
        contact = LIBRARY['attacks'][name]['contact']
        left, right, draw, visible, torso = bow_pose(name, t, contact, left, right)
        anatomical(bones['spine'], torso)
        anatomical(bones['head'], (-.03, 0, -torso[2]*.6))
    elif name == 'mine':
        right = curve(t, [(0, (-.2, -.28, 1.0)), (.40, (-.14, -.06, 1.48)), (.60, (-.15, -.50, 1.03)), (.69, (-.15, -.48, 1.02)), (1, (-.2, -.28, 1.0))])
        angles = curve(t, [(0, (.45, 0, 0)), (.40, (-.48, 0, 0)), (.60, (1.08, 0, 0)), (.69, (1.02, 0, 0)), (1, (.45, 0, 0))])
        anatomical(bones['spine'], (.04+.14*math.sin(t*math.pi)**2, 0, .025*math.sin(t*math.tau)))
    left_grip = arm(obj, 'l', left)
    right_grip = arm(obj, 'r', right, angles)
    if kind == 'miner' and name == 'mine':
        arm(obj, 'l', right_grip+Euler(angles, 'XYZ').to_quaternion() @ Vector((0, 0, .20)), angles)
    if kind == 'archer':
        nock(obj, left_grip, right_grip, draw, visible)
    else:
        bones['arrow_socket'].scale = (.001,)*3


def fp_pose(obj, kind, name, t):
    bones = obj.pose.bones
    grips = {side: bones['hand_'+side].bone.head_local.copy() for side, _ in SIDES}
    contact = LIBRARY['attacks'].get(name, {}).get('contact', .4)
    targets, angles, draw, visible = first_person_pose(name, t, contact, grips)
    if name.startswith('hit_'):
        a = math.sin(t*math.pi)*math.exp(-t*2)
        for side in targets:
            targets[side] += Vector((.02*a, -.06*a, -.045*a))
            angles[side].x += .15*a
    for side, _ in SIDES:
        fore, hand = bones['forearm_'+side], bones['hand_'+side]
        elbow = fore.bone.head_local.copy()
        orient(fore, elbow, targets[side])
        fore.scale.y = (targets[side]-elbow).length/fore.bone.length
        bpy.context.view_layer.update()
        matrix = Euler(angles[side], 'XYZ').to_matrix().to_4x4() @ hand.bone.matrix_local
        matrix.translation = targets[side]
        hand.matrix = matrix
        bpy.context.view_layer.update()
    if kind == 'archer':
        nock(obj, targets['l'], targets['r'], draw, visible, forward=1)
    else:
        bones['arrow_socket'].scale = (.001,)*3


def create_animations(obj, kind, first_person=False):
    names = {'idle': 2.4}
    if not first_person:
        names.update({name: spec['seconds'] for name, spec in LIBRARY['locomotion'].items() if name != 'runThreshold'})
        if kind == 'miner':
            names['mine'] = 1.5
        names.update({name: LIBRARY['deathSeconds'] for name in ['death_front', 'death_back', 'death_left', 'death_right']})
    names.update({name: LIBRARY['hitSeconds'] for name in ['hit_front', 'hit_back', 'hit_left', 'hit_right']})
    prefix = 'bow_' if kind == 'archer' else 'melee_' if kind == 'swordsman' else None
    names.update({name: spec['seconds'] for name, spec in LIBRARY['attacks'].items() if prefix and name.startswith(prefix)})
    obj.animation_data_clear()
    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)
    bpy.context.scene.render.fps = 30
    for name, seconds in names.items():
        obj.animation_data_create()
        obj.animation_data.action = None
        frames = seconds*30
        samples = set(range(math.floor(frames)+1)) | {frames}
        if name in LIBRARY['attacks']:
            # An exact contact key prevents interpolation into the released
            # string/hidden arrow before the simulation's release timestamp.
            samples.add(frames*LIBRARY['attacks'][name]['contact'])
        for frame in sorted(samples):
            clear_pose(obj)
            phase = frame/frames
            contact = LIBRARY['attacks'].get(name, {}).get('contact')
            if contact is not None and abs(phase-contact) < 1e-8:
                phase = contact
            (fp_pose if first_person else humanoid_pose)(obj, kind, name, phase)
            for bone in obj.pose.bones:
                for path in ['rotation_quaternion', 'location', 'scale']:
                    bone.keyframe_insert(data_path=path, frame=frame+1, group=bone.name)
        action = obj.animation_data.action
        action.name = name
        action.use_fake_user = True
        # The poses already contain eased choreography. Linear keys preserve
        # exact contact times and avoid Bezier overshoot at planted feet.
        for layer in action.layers:
            for strip in layer.strips:
                for bag in strip.channelbags:
                    for channel in bag.fcurves:
                        for key in channel.keyframe_points:
                            key.interpolation = 'LINEAR'
        track = obj.animation_data.nla_tracks.new()
        track.name = name
        strip = track.strips.new(name, 1, action)
        strip.action_frame_start, strip.action_frame_end = 1, frames+1
        track.mute = True
        if name in LIBRARY['attacks']:
            marker = action.pose_markers.new('CONTACT — simulation windup')
            marker.frame = 1+round(frames*LIBRARY['attacks'][name]['contact'])
        print('BAKED', kind, name, frames+1)
    obj.animation_data.action = None
    clear_pose(obj)
    bpy.context.scene.frame_set(1)
    obj['rig_contract'] = 'stick-war-humanoid-v2' if not first_person else 'stick-war-first-person-v2'
    obj['animation_notes'] = 'Actions animate bones only. Bind replacement meshes to this rig; keep names and rest transforms.'
