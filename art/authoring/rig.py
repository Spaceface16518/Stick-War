"""Shared skeleton contract; bind replacement meshes to these bones."""
import bpy

def rig():
    arm = bpy.data.armatures.new('Common humanoid skeleton')
    obj = bpy.data.objects.new('Humanoid', arm)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    specs = [
        ('root',(0,0,0),(0,0,.2),None),
        ('pelvis',(0,0,.85),(0,0,1.03),'root'),
        ('spine',(0,0,1.03),(0,0,1.38),'pelvis'),
        ('head',(0,0,1.43),(0,0,1.75),'spine'),
        ('eye_socket',(0,-.12,1.69),(0,-.23,1.69),'head'),
    ]
    for side, sign in [('l',1),('r',-1)]:
        specs.extend([
            ('upper_arm_'+side,(sign*.29,0,1.35),(sign*.41,0,1.03),'spine'),
            ('forearm_'+side,(sign*.41,0,1.03),(sign*.43,-.035,.78),'upper_arm_'+side),
            ('hand_'+side,(sign*.43,-.035,.78),(sign*.43,-.045,.68),'forearm_'+side),
            ('thigh_'+side,(sign*.14,0,.88),(sign*.15,0,.48),'pelvis'),
            ('shin_'+side,(sign*.15,0,.48),(sign*.15,0,.13),'thigh_'+side),
            ('foot_'+side,(sign*.15,0,.13),(sign*.15,-.18,.10),'shin_'+side),
            ('toe_'+side,(sign*.15,-.10,.10),(sign*.15,-.23,.10),'foot_'+side),
        ])
    specs.extend([
        ('weapon_socket',(-.43,-.045,.755),(-.43,-.15,.755),'hand_r'),
        ('bow_socket',(.43,-.045,.755),(.43,-.15,.755),'hand_l'),
        ('bow_upper',(.43,-.045,.755),(.43,.135,1.275),'bow_socket'),
        ('bow_lower',(.43,-.045,.755),(.43,.135,.235),'bow_socket'),
        ('string_nock',(.43,.135,.755),(.43,.135,.855),'bow_socket'),
        ('arrow_socket',(.43,.135,.755),(.43,.035,.755),'string_nock')
    ])
    for name, head, tail, parent in specs:
        bone = arm.edit_bones.new(name)
        bone.head, bone.tail = head, tail
        if parent:
            bone.parent = arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')
    obj.show_in_front = True
    return obj
