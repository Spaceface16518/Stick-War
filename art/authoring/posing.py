"""Bake anatomical targets into ordinary bone actions; no runtime IK dependency."""
import math
import bpy
from mathutils import Vector, Matrix, Euler


def curve(t, keys):
    for (a, va), (b, vb) in zip(keys, keys[1:]):
        if t <= b:
            f = max(0, min(1, (t-a)/(b-a)))
            f = f*f*(3-2*f)
            if len(va) == 1:
                return (va[0]+(vb[0]-va[0])*f,)
            return Vector(va).lerp(Vector(vb), f)
    return keys[-1][1] if len(keys[-1][1]) == 1 else Vector(keys[-1][1])


def clear_pose(obj):
    for bone in obj.pose.bones:
        bone.rotation_mode = 'QUATERNION'
        bone.rotation_quaternion = (1, 0, 0, 0)
        bone.location = (0, 0, 0)
        bone.scale = (1, 1, 1)
    bpy.context.view_layer.update()


def anatomical(bone, angles=(0, 0, 0), offset=(0, 0, 0)):
    """Rotate in the skeleton's anatomical axes, respecting its posed parent."""
    head = bone.bone.head_local
    parent = bone.parent
    parent_delta = parent.matrix @ parent.bone.matrix_local.inverted() if parent else Matrix.Identity(4)
    rotation = Euler(angles, 'XYZ').to_matrix().to_4x4()
    bone.matrix = parent_delta @ Matrix.Translation(Vector(offset)+head) @ rotation @ Matrix.Translation(-head) @ bone.bone.matrix_local
    bpy.context.view_layer.update()


def orient(bone, head, tail):
    rest = bone.bone.matrix_local
    rotation = (bone.bone.tail_local-bone.bone.head_local).rotation_difference(tail-head)
    matrix = rotation.to_matrix().to_4x4() @ rest
    matrix.translation = head
    bone.matrix = matrix
    bpy.context.view_layer.update()


def limb(first, second, target, pole):
    start = first.head.copy()
    delta = Vector(target)-start
    axis = delta.normalized()
    a, b = first.bone.length, second.bone.length
    distance = max(.025, min(delta.length, a+b-.0005))
    end = start+axis*distance
    along = (a*a-b*b+distance*distance)/(2*distance)
    pole = Vector(pole)
    bend = (pole-axis*pole.dot(axis)).normalized()
    joint = start+axis*along+bend*math.sqrt(max(0, a*a-along*along))
    orient(first, start, joint)
    orient(second, joint, end)
    return end


def arm(obj, side, grip, angles=(0, 0, 0)):
    bones = obj.pose.bones
    upper, fore, hand = [bones[p+'_'+side] for p in ['upper_arm', 'forearm', 'hand']]
    rotation = Euler(angles, 'XYZ').to_quaternion()
    palm = Vector((.43 if side == 'l' else -.43, -.045, .755))
    wrist = Vector(grip)-rotation @ (palm-hand.bone.head_local)
    wrist = limb(upper, fore, wrist, (1 if side == 'l' else -1, .7, -.2))
    matrix = rotation.to_matrix().to_4x4() @ hand.bone.matrix_local
    matrix.translation = wrist
    hand.matrix = matrix
    bpy.context.view_layer.update()
    return wrist+rotation @ (palm-hand.bone.head_local)


def leg(obj, side, at, pitch=0, toe=0):
    bones = obj.pose.bones
    ankle = limb(bones['thigh_'+side], bones['shin_'+side], at, (0, -1, .05))
    foot = bones['foot_'+side]
    matrix = Euler((pitch, 0, 0), 'XYZ').to_matrix().to_4x4() @ foot.bone.matrix_local
    matrix.translation = ankle
    foot.matrix = matrix
    bpy.context.view_layer.update()
    anatomical(bones['toe_'+side], (toe, 0, 0))


def nock(obj, bow_grip, right_grip, draw, visible, forward=-1):
    bones = obj.pose.bones
    # Bend the two limbs around the grip while each end of the string follows
    # its own bone. The midpoint stays in the drawing hand until release.
    for side, sign in [('upper', 1), ('lower', -1)]:
        anatomical(bones['bow_'+side], (forward*sign*.09*draw, 0, 0))
    center = Vector(bow_grip)+Vector((0, -forward*.18, 0))
    center = center.lerp(Vector(right_grip), draw)
    bone = bones['string_nock']
    matrix = bones['bow_socket'].matrix @ bones['bow_socket'].bone.matrix_local.inverted() @ bone.bone.matrix_local
    matrix.translation = center
    bone.matrix = matrix
    bones['arrow_socket'].scale = (1, 1, 1) if visible else (.001, .001, .001)
    bpy.context.view_layer.update()
