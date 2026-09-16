"""Original tabletop models. Run with Blender 5; no third-party Python packages.
Blender source: Z up, forward -Y. GLB: metres, Y up, forward +Z.
Art is deliberately reproducible, editable geometry, not a runtime character editor.
"""
import bpy
import math
import json
import random
import sys
from pathlib import Path
from mathutils import Vector, Matrix, Euler

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'art/export'
SRC = ROOT / 'art/sources'
TEX = ROOT / 'art/textures'
for directory in (OUT, SRC, TEX):
    directory.mkdir(parents=True, exist_ok=True)
PALETTE = [
    (0.68, .44, .29), (.83, .62, .43), (.29, .18, .105), (.12, .10, .08),
    (.47, .51, .49), (.72, .74, .65), (.70, .53, .24), (.92, .87, .72),
    (.33, .40, .21), (.49, .55, .32), (.60, .59, .45), (.76, .73, .57),
    (.24, .32, .20), (.20, .40, .57), (.63, .24, .14), (.91, .91, .85)
]
PARTS = []
BONE = None
MATS = []
RNG = random.Random(734)

def reset():
    global PARTS, MATS
    bpy.ops.wm.read_factory_settings(use_empty=True)
    PARTS = []
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.render.fps = 30
    scene.world = bpy.data.worlds.new('Daylight')
    scene.world.color = (.3, .38, .43)
    import numpy as np
    image = bpy.data.images.new('Tabletop paint 1K', width=1024, height=1024)
    pixels = np.ones((1024, 1024, 4), dtype=np.float32)
    noise = np.tile(np.random.default_rng(24).integers(-2, 3, (32, 32, 1)), (8, 8, 1)) / 255
    yy, xx = np.mgrid[0:256, 0:256]
    detail = np.round((1 - .07 * ((xx - 128)**2 + (yy - 128)**2) / 32768) * 24) / 24
    for i, color in enumerate(PALETTE):
        x, y = i % 4 * 256, i // 4 * 256
        pixels[y:y+256, x:x+256, :3] = np.clip(np.array(color)[None,None,:] * detail[:,:,None] + noise, 0, 1)
    image.pixels.foreach_set(pixels.ravel())
    image.filepath_raw = str(TEX / 'tabletop-palette.png')
    image.file_format = 'PNG'
    image.save()
    image.pack()
    MATS = []
    for name in ['Painted palette', 'Team cloth']:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get('Principled BSDF')
        bsdf.inputs['Roughness'].default_value = .95 if name == 'Team cloth' else .8
        tex = mat.node_tree.nodes.new('ShaderNodeTexImage')
        tex.image = image
        mat.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
        MATS.append(mat)

def finish(obj, name, tile=1, bone=None, team=False, smooth=True):
    obj.name = name
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(MATS[1 if team else 0])
    uv = obj.data.uv_layers.active or obj.data.uv_layers.new(name='Paint UV')
    uv.name = 'Paint UV'
    tile = 15 if team else tile
    tx, ty = tile % 4, tile // 4
    for poly in obj.data.polygons:
        poly.use_smooth = smooth
        for j, li in enumerate(poly.loop_indices):
            # Each polygon stays inside its swatch with gently varied painted detail.
            a = 2 * math.pi * j / len(poly.loop_indices)
            uv.data[li].uv = ((tx + .5 + .27 * math.cos(a))/4, (ty + .5 + .27 * math.sin(a))/4)
    if bone:
        group = obj.vertex_groups.new(name=bone)
        group.add(list(range(len(obj.data.vertices))), 1, 'REPLACE')
    PARTS.append(obj)
    return obj

def ell(name, at, size, tile=1, bone=None, team=False, seg=10, rings=7, smooth=True):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=seg, ring_count=rings, location=at)
    o = bpy.context.object
    o.scale = size
    return finish(o, name, tile, bone, team, smooth)

def cube(name, at, size, tile=2, bone=None, team=False, bevel=0):
    bpy.ops.mesh.primitive_cube_add(size=1, location=at)
    o = bpy.context.object
    o.scale = size
    finish(o, name, tile, bone, team, False)
    if bevel:
        mod = o.modifiers.new('Rounded hand-carved edges', 'BEVEL')
        mod.width = bevel
        mod.segments = 1
        bpy.context.view_layer.objects.active = o
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return o

def rod(name, a, b, radius, tile=2, bone=None, team=False, vertices=8, end_radius=None):
    a, b = Vector(a), Vector(b)
    bpy.ops.mesh.primitive_cone_add(vertices=vertices, radius1=radius,
        radius2=radius if end_radius is None else end_radius, depth=(b-a).length, location=(a+b)*.5)
    o = bpy.context.object
    o.rotation_mode = 'QUATERNION'
    o.rotation_quaternion = (b-a).to_track_quat('Z', 'Y')
    return finish(o, name, tile, bone, team)

def join(name, parts=None):
    parts = parts or list(PARTS)
    bpy.ops.object.select_all(action='DESELECT')
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    obj = bpy.context.object
    obj.name = name
    bpy.context.scene.cursor.location = (0,0,0)
    bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
    return obj

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
        ])
    specs.extend([
        ('weapon_socket',(-.43,-.045,.755),(-.43,-.15,.755),'hand_r'),
        ('bow_socket',(.43,-.045,.755),(.43,-.15,.755),'hand_l'),
        ('string_nock',(.43,.135,.755),(.43,.135,.855),'root'),
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

def sword(center=(-.43,-.045,.755), bone='weapon_socket', scale=1, forward=1):
    c = Vector(center)
    def p(x,y,z): return c + Vector((x,y*forward,z))*scale
    rod('Wrapped grip', p(0,0,-.1), p(0,0,.12), .034*scale, 2, bone)
    ell('Pommel',p(0,0,-.13),(.05*scale,)*3,6,bone,seg=8,rings=5)
    rod('Steel crossguard',p(-.15,0,.13),p(.15,0,.13),.026*scale,6,bone)
    # Diamond-section blade, pointed tip, rather than a cuboid.
    verts = [p(-.055,0,.16),p(0,-.025,.16),p(.055,0,.16),p(0,.025,.16),
             p(-.042,-.16,.75),p(0,-.185,.75),p(.042,-.16,.75),p(0,-.135,.75),p(0,-.20,.9)]
    faces=[(0,1,5,4),(1,2,6,5),(2,3,7,6),(3,0,4,7),(4,5,8),(5,6,8),(6,7,8),(7,4,8),(3,2,1,0)]
    mesh=bpy.data.meshes.new('Forged blade');mesh.from_pydata(verts,[],faces);mesh.update()
    ob=bpy.data.objects.new('Sword blade',mesh);bpy.context.collection.objects.link(ob)
    finish(ob,'Sword blade',5,bone,smooth=False)

def string_segment(a, b, bone_a, bone_b):
    # Each end follows its own attachment, so drawing the string stretches the
    # two halves without moving the bow out of the hand.
    a,b=Vector(a),Vector(b)
    verts=[a+Vector((dx,0,dz)) for dx,dz in [(-.004,0),(.004,0),(0,.004)]]
    verts += [b+Vector((dx,0,dz)) for dx,dz in [(-.004,0),(.004,0),(0,.004)]]
    mesh=bpy.data.meshes.new('Linen bowstring');mesh.from_pydata(verts,[],[(0,1,4,3),(1,2,5,4),(2,0,3,5)]);mesh.update()
    ob=bpy.data.objects.new('Linen bowstring',mesh);bpy.context.collection.objects.link(ob)
    finish(ob,ob.name,7)
    ob.vertex_groups.new(name=bone_a).add([0,1,2],1,'REPLACE')
    ob.vertex_groups.new(name=bone_b).add([3,4,5],1,'REPLACE')

def bow(center=(.43,-.045,.755), bone='bow_socket', scale=1, forward=1):
    c=Vector(center)
    points=[]
    for i in range(9):
        t=-1+i/4
        points.append(c+Vector((0,.18*forward*t*t,t*.52))*scale)
    for i in range(8):rod('Yew bow limb',points[i],points[i+1],.024*scale,2,bone,end_radius=.018*scale)
    if bone:
        nock=c+Vector((0,.18*forward,0))*scale
        string_segment(points[0],nock,bone,'string_nock')
        string_segment(nock,points[-1],'string_nock',bone)
        rod('Nocked arrow',nock,nock+Vector((0,-.85*forward,0)),.008,2,'arrow_socket',vertices=6)
        ell('Arrowhead',nock+Vector((0,-.86*forward,0)),(.025,.06,.015),4,'arrow_socket',seg=6,rings=4)
    else:
        rod('Linen string',points[0],points[-1],.005*scale,7,bone,vertices=4)
    rod('Leather bow grip',c+Vector((0,0,-.07))*scale,c+Vector((0,0,.07))*scale,.032*scale,3,bone)

def character(kind):
    reset()
    # Broad shoulders and readable hands; roughly six heads tall.
    ell('Quilted tunic',(0,0,1.16),(.29,.16,.32),bone='spine',team=True,seg=12,rings=8)
    ell('Tunic skirt',(0,.015,.9),(.29,.17,.15),bone='pelvis',team=True)
    cube('Waist belt',(0,-.006,1.00),(.56,.34,.075),2,'pelvis',bevel=.02)
    cube('Brass buckle',(0,-.185,1.00),(.1,.025,.07),6,'pelvis',bevel=.01)
    ell('Neck',(0,0,1.45),(.08,.08,.11),1,'head')
    ell('Face',(0,-.005,1.64),(.145,.126,.19),1,'head',seg=12,rings=8)
    ell('Jaw',(0,-.045,1.54),(.107,.095,.085),0,'head')
    ell('Nose',(0,-.132,1.64),(.036,.049,.055),1,'head',seg=8,rings=5)
    for side, sign in [('l',1),('r',-1)]:
        ell('Ear '+side,(sign*.14,-.002,1.64),(.035,.035,.06),0,'head',seg=8,rings=5)
        ell('Eye '+side,(sign*.056,-.119,1.685),(.029,.012,.017),7,'head',seg=8,rings=5)
        ell('Pupil '+side,(sign*.056,-.130,1.685),(.012,.005,.012),3,'head',seg=6,rings=4)
        rod('Brow '+side,(sign*.028,-.13,1.713),(sign*.087,-.11,1.706),.009,2,'head',vertices=5)
        ell('Sleeve '+side,(sign*.33,.005,1.28),(.115,.13,.19),bone='upper_arm_'+side,team=True)
        rod('Upper arm '+side,(sign*.33,0,1.27),(sign*.41,0,1.03),.087,0,'upper_arm_'+side,end_radius=.075)
        ell('Elbow '+side,(sign*.41,0,1.035),(.082,.083,.08),0,'forearm_'+side)
        rod('Forearm '+side,(sign*.41,0,1.04),(sign*.43,-.035,.8),.075,1,'forearm_'+side,end_radius=.05)
        rod('Leather bracer '+side,(sign*.43,-.023,.9),(sign*.43,-.035,.81),.079,2,'forearm_'+side,end_radius=.064)
        ell('Hand '+side,(sign*.43,-.045,.755),(.062,.057,.078),1,'hand_'+side,seg=8,rings=6)
        ell('Thumb '+side,(sign*.383,-.08,.762),(.024,.036,.04),0,'hand_'+side,seg=6,rings=4)
        rod('Trouser thigh '+side,(sign*.14,.015,.85),(sign*.15,0,.48),.112,3,'thigh_'+side,end_radius=.087)
        ell('Knee '+side,(sign*.15,-.025,.46),(.088,.098,.085),2,'shin_'+side)
        rod('Boot shaft '+side,(sign*.15,0,.44),(sign*.15,0,.14),.097,2,'shin_'+side,end_radius=.08)
        ell('Boot '+side,(sign*.15,-.055,.11),(.099,.185,.103),2,'foot_'+side)
        cube('Boot sole '+side,(sign*.15,-.067,.034),(.205,.32,.058),3,'foot_'+side,bevel=.014)
    if kind=='swordsman':
        ell('Open steel helmet',(0,.01,1.76),(.17,.146,.106),4,'head',seg=12,rings=7)
        for sign in [-1,1]:
            cube('Helmet cheek guard',(sign*.139,.012,1.64),(.032,.14,.17),4,'head',bevel=.014)
            ell('Steel shoulder',(sign*.325,.012,1.37),(.135,.15,.084),4,'upper_arm_'+('l' if sign==1 else 'r'))
        rod('Helmet brow rim',(-.145,-.09,1.746),(.145,-.09,1.746),.024,5,'head')
        cube('Chest leather harness',(0,-.161,1.235),(.057,.018,.36),2,'spine')
        sword()
    elif kind=='miner':
        ell('Wool cap',(0,.016,1.77),(.157,.142,.105),2,'head')
        ell('Beard',(0,-.10,1.53),(.10,.064,.085),2,'head')
        cube('Leather apron',(0,-.166,1.02),(.37,.028,.47),2,'spine',bevel=.02)
        ell('Canvas ore sack',(0,.23,1.14),(.21,.145,.27),6,'spine')
        rod('Sack strap',(-.17,-.15,1.38),(.15,-.18,1.02),.025,3,'spine')
        rod('Pickaxe handle',(-.43,-.045,.54),(-.43,-.045,1.42),.027,2,'weapon_socket')
        rod('Pickaxe head',(-.76,-.045,1.42),(-.1,-.045,1.42),.042,4,'weapon_socket',end_radius=.008)
    else:
        ell('Archer hood',(0,.045,1.70),(.167,.144,.196),bone='head',team=True)
        # An exposed front face is layered forward of the hood.
        ell('Hood opening face',(0,-.055,1.64),(.131,.105,.17),1,'head',seg=12,rings=8)
        for sign in [-1,1]:
            ell('Visible eye',(sign*.053,-.152,1.68),(.022,.008,.014),3,'head',seg=6,rings=4)
        ell('Archer nose',(0,-.16,1.63),(.033,.034,.046),1,'head',seg=8,rings=5)
        rod('Quiver',( .14,.23,1.05),(.19,.22,1.5),.084,2,'spine')
        for i in range(4):
            x=.14+(i%2)*.055;y=.19+(i//2)*.045
            rod('Spare arrow',(x,y,1.21),(x+.03,y,1.7),.009,2,'spine',vertices=5)
            cube('Arrow fletching',(x+.03,y,1.66),(.047,.018,.08),7,'spine')
        rod('Quiver strap',(.17,-.145,1.4),(-.21,-.18,1.04),.027,2,'spine')
        bow()
    mesh=join(kind.title()+' painted model')
    skeleton=rig()
    mesh.parent=skeleton
    mod=mesh.modifiers.new('Humanoid skin','ARMATURE');mod.object=skeleton
    create_animations(skeleton,kind)
    save_export(kind)
    # Commander LOD preserves the same skeleton, clips, sockets and material slots.
    bpy.context.view_layer.objects.active=mesh
    dec=mesh.modifiers.new('Commander geometry reduction','DECIMATE');dec.ratio=.40
    bpy.ops.object.modifier_move_up(modifier=dec.name)
    bpy.ops.object.modifier_apply(modifier=dec.name)
    mesh.data.validate()
    mesh.data.update()
    export(kind+'_lod')

def eased_keys(t, keys):
    for (a,va),(b,vb) in zip(keys,keys[1:]):
        if t <= b:
            f=max(0,min(1,(t-a)/(b-a)));f=f*f*(3-2*f)
            return Vector(va).lerp(Vector(vb),f)
    return Vector(keys[-1][1])

def orient_bone(bone, head, tail):
    rest=bone.bone.matrix_local
    q=(bone.bone.tail_local-bone.bone.head_local).rotation_difference(tail-head)
    matrix=q.to_matrix().to_4x4() @ rest
    matrix.translation=head
    bone.matrix=matrix
    bpy.context.view_layer.update()

def pose_arm(obj, side, grip, angles=(0,0,0)):
    bones=obj.pose.bones
    upper,fore,hand=[bones[p+'_'+side] for p in ['upper_arm','forearm','hand']]
    q=Euler(angles,'XYZ').to_quaternion()
    # The tool's origin is the palm, not the forearm pivot. Solve to that
    # contact point, then orient the wrist independently of elbow bending.
    palm=Vector((.43 if side=='l' else -.43,-.045,.755))
    wrist=Vector(grip)-q @ (palm-hand.bone.head_local)
    shoulder=upper.head.copy()
    delta=wrist-shoulder;distance=delta.length
    axis=delta.normalized();a=upper.bone.length;b=fore.bone.length
    distance=max(.03,min(distance,a+b-.002));wrist=shoulder+axis*distance
    along=(a*a-b*b+distance*distance)/(2*distance)
    pole=Vector((1 if side=='l' else -1,.65,-.3))
    bend=(pole-axis*pole.dot(axis)).normalized()
    elbow=shoulder+axis*along+bend*math.sqrt(max(0,a*a-along*along))
    orient_bone(upper,shoulder,elbow)
    orient_bone(fore,elbow,wrist)
    matrix=q.to_matrix().to_4x4() @ hand.bone.matrix_local
    matrix.translation=wrist;hand.matrix=matrix
    bpy.context.view_layer.update()
    return wrist+q @ (palm-hand.bone.head_local)

def create_animations(obj,kind):
    bones=obj.pose.bones
    names=['idle','walk','carry','mine','melee_attack','bow_attack','hit','death']
    for name in names:
        duration={'idle':60,'walk':30,'carry':36,'mine':45,'melee_attack':60,'bow_attack':60,'hit':12,'death':36}[name]
        obj.animation_data_create();obj.animation_data.action=None
        for frame in range(1,duration+2):
            t=(frame-1)/duration;a=math.sin(t*math.tau)
            for b in bones:
                b.rotation_mode='QUATERNION';b.rotation_quaternion=(1,0,0,0);b.location=(0,0,0);b.scale=(1,1,1)
            def rotate(bone,angles):bones[bone].rotation_quaternion=Euler(angles,'XYZ').to_quaternion()
            rotate('spine',(.012*a,0,.008*a))
            rotate('head',(-.008*a,0,-.006*a))
            if name in ('walk','carry'):
                for side,sign in [('l',1),('r',-1)]:
                    stride=a*sign
                    rotate('thigh_'+side,(.42*stride,0,0))
                    rotate('shin_'+side,(max(0,-stride)*.6,0,0))
                    rotate('foot_'+side,(max(0,-stride)*.22,0,0))
                bones['pelvis'].location.y=.012*(1-math.cos(t*math.tau*2))
                rotate('spine',(.055 if name=='carry' else .025,.025*a,.02*a))
                rotate('head',(-.025,-.018*a,-.015*a))
            swing=.035*a if name in ('walk','carry') else .008*a
            left=(.35,-.06+swing,.84)
            right=(-.34,-.22-swing,1.04)
            angles=(.25,0,-.1)
            if kind=='miner':
                right=(-.36,-.15-swing,.87);angles=(.18,0,.05)
            elif kind=='archer':
                left=(.34,-.16-swing,.95);right=(-.30,-.16+swing,1.04);angles=(0,0,0)
            if name=='melee_attack' and kind=='swordsman':
                right=eased_keys(t,[(0,right),(.21,(-.42,.01,1.54)),(.35,(-.14,-.52,1.22)),(.50,(.10,-.40,.96)),(.78,(-.25,-.27,1.02)),(1,right)])
                angles=eased_keys(t,[(0,angles),(.21,(-.65,-.20,-.25)),(.35,(1.35,-.12,-.22)),(.50,(1.6,.30,.22)),(.78,(.4,0,-.1)),(1,angles)])
                twist=eased_keys(t,[(0,(0,0,0)),(.21,(.03,0,-.20)),(.35,(.10,0,.16)),(.55,(.06,0,.23)),(1,(0,0,0))])
                rotate('spine',twist);rotate('head',(-twist.x*.4,0,-twist.z*.5))
                rotate('thigh_l',(-.09*math.sin(t*math.pi),0,0))
                left=(.33,-.19,1.1)
            if name=='mine' and kind=='miner':
                right=eased_keys(t,[(0,(-.2,-.28,1.0)),(.42,(-.14,-.06,1.48)),(.62,(-.15,-.50,1.03)),(.73,(-.15,-.48,1.02)),(1,(-.2,-.28,1.0))])
                angles=eased_keys(t,[(0,(.45,0,0)),(.42,(-.48,0,0)),(.62,(1.08,0,0)),(.73,(1.02,0,0)),(1,(.45,0,0))])
                left=Vector(right)+Euler(angles,'XYZ').to_quaternion() @ Vector((0,0,.20))
                rotate('spine',(.04+.13*math.sin(t*math.pi)**2,0,0))
            if name=='bow_attack' and kind=='archer':
                left=eased_keys(t,[(0,left),(.18,(.22,-.51,1.42)),(.35,(.22,-.51,1.42)),(.46,(.22,-.49,1.42)),(.72,(.32,-.23,1.10)),(1,left)])
                right=eased_keys(t,[(0,right),(.12,(.22,-.34,1.38)),(.29,(.22,-.06,1.42)),(.35,(.22,-.06,1.42)),(.41,(.15,.025,1.45)),(.64,(-.06,.12,1.48)),(.82,(.19,-.10,1.08)),(1,right)])
                rotate('spine',(.025,0,.06*math.sin(t*math.pi)))
                rotate('head',(-.025,0,-.06*math.sin(t*math.pi)))
            if name=='hit':
                rotate('spine',(-.12*math.sin(t*math.pi),0,.05*math.sin(t*math.pi)))
                rotate('head',(.07*math.sin(t*math.pi),0,0))
            if name=='death':
                f=min(1,t/0.75);f=f*f*(3-2*f)
                rotate('root',(1.48*f,0,.12*f));bones['root'].location.z=-.1*f
            bpy.context.view_layer.update()
            # Death keeps the natural falling hierarchy. Other clips use
            # baked analytic two-bone IK for stable grips and elbow arcs.
            if name!='death':
                left_grip=pose_arm(obj,'l',left,(0,0,0))
                right_grip=pose_arm(obj,'r',right,angles)
                if kind=='miner' and name=='mine':
                    left_grip=pose_arm(obj,'l',right_grip+Euler(angles,'XYZ').to_quaternion() @ Vector((0,0,.20)),angles)
                if kind=='archer':
                    nock=left_grip+Vector((0,.18,0))
                    if name=='bow_attack' and .10<=t<=.35:nock=right_grip.copy()
                    elif name=='bow_attack' and .35<t<.43:
                        f=(t-.35)/.08;nock=nock.lerp(Vector((.22,-.06,1.42)),(1-f)**2*.25*math.cos(f*math.tau*2))
                    mat=bones['string_nock'].bone.matrix_local.copy();mat.translation=nock;bones['string_nock'].matrix=mat
            bones['arrow_socket'].scale=(1,1,1) if name=='bow_attack' and .10<=t<=.35 else (.001,.001,.001)
            for b in bones:
                b.keyframe_insert(data_path='rotation_quaternion',frame=frame,group=b.name)
                b.keyframe_insert(data_path='location',frame=frame,group=b.name)
                b.keyframe_insert(data_path='scale',frame=frame,group=b.name)
        action=obj.animation_data.action;action.name=name;action.use_fake_user=True
        track=obj.animation_data.nla_tracks.new();track.name=name
        strip=track.strips.new(name,1,action);strip.action_frame_start=1;strip.action_frame_end=duration+1
        track.mute=True
    obj.animation_data.action=None
    for b in bones:b.rotation_quaternion=(1,0,0,0);b.location=(0,0,0);b.scale=(1,1,1)
    bpy.context.scene.frame_set(1)

def export(name):
    bpy.ops.export_scene.gltf(filepath=str(OUT/(name+'.glb')),export_format='GLB',
        export_yup=True,export_animations=True,export_animation_mode='ACTIONS',
        export_skins=True,export_morph=False,export_cameras=False,export_lights=False,
        export_extras=True,export_apply=False)

def save_export(name):
    bpy.context.scene['asset_id']=name
    bpy.context.scene['convention']='Metres, feet at origin, glTF +Z forward, shared named sockets'
    bpy.ops.wm.save_as_mainfile(filepath=str(SRC/(name+'.blend')))
    export(name)

def first_person(kind):
    reset()
    def p(x,y,z):return Vector((x,-z,y))
    grips={'l':p(-.24,-.27,-.46),'r':p(.24,-.27,-.46)}
    if kind=='archer':grips['l']=p(-.24,-.22,-.58)
    elbows={side:p(sign*.38,-.49,-.05) for side,sign in [('l',-1),('r',1)]}
    for side,sign in [('l',-1),('r',1)]:
        wrist=grips[side];elbow=elbows[side]
        mid=elbow.lerp(wrist,.48)
        rod('Sleeve '+side,elbow,mid,.075,bone='forearm_'+side,team=True,end_radius=.065)
        rod('Forearm '+side,mid,wrist,.059,1,'forearm_'+side,end_radius=.044)
        rod('Bracer '+side,elbow.lerp(wrist,.80),wrist,.060,2,'forearm_'+side,end_radius=.055)
        ell('Hand '+side,wrist,(.06,.062,.05),1,'hand_'+side,seg=10,rings=6)
        for finger in range(3):
            ell('Knuckle',wrist+Vector(((finger-1)*.024,.035,0)),(.014,.022,.016),0,'hand_'+side,seg=6,rings=4)
    if kind=='swordsman':sword(center=grips['r'],bone='weapon_socket',scale=.66,forward=-1)
    else:bow(center=grips['l'],bone='bow_socket',scale=.95,forward=-1)
    mesh=join('First person '+kind)
    arm=bpy.data.armatures.new('First person arms');obj=bpy.data.objects.new('FirstPersonRig',arm)
    bpy.context.collection.objects.link(obj);bpy.context.view_layer.objects.active=obj;obj.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    specs=[('root',Vector((0,0,0)),Vector((0,0,.1)),None)]
    for side in ['l','r']:
        specs += [('forearm_'+side,elbows[side],grips[side],'root'),('hand_'+side,grips[side],grips[side]+Vector((0,.1,0)),'forearm_'+side)]
    nock=grips['l']+Vector((0,-.18*.95,0))
    specs += [('weapon_socket',grips['r'],grips['r']+Vector((0,.1,0)),'hand_r'),('bow_socket',grips['l'],grips['l']+Vector((0,.1,0)),'hand_l'),('string_nock',nock,nock+Vector((0,0,.1)),'root'),('arrow_socket',nock,nock+Vector((0,.1,0)),'string_nock')]
    for name,head,tail,parent in specs:
        b=arm.edit_bones.new(name);b.head=head;b.tail=tail
        if parent:b.parent=arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')
    mesh.parent=obj;mod=mesh.modifiers.new('First person skin','ARMATURE');mod.object=obj
    bones=obj.pose.bones
    for name in ['idle','melee_attack','bow_attack']:
        obj.animation_data_create();obj.animation_data.action=None
        for frame in range(1,62):
            t=(frame-1)/60
            for b in bones:
                b.rotation_mode='QUATERNION';b.rotation_quaternion=(1,0,0,0);b.location=(0,0,0);b.scale=(1,1,1)
            targets={s:g.copy() for s,g in grips.items()};angles={'l':(0,0,0),'r':(0,0,0)}
            if name=='melee_attack' and kind=='swordsman':
                targets['r']=eased_keys(t,[(0,grips['r']),(.21,p(.39,-.10,-.35)),(.35,p(-.04,-.17,-.65)),(.50,p(-.26,-.32,-.48)),(.8,p(.15,-.30,-.42)),(1,grips['r'])])
                angles['r']=eased_keys(t,[(0,(0,0,0)),(.21,(.3,.15,-.3)),(.35,(-1.30,-.2,.3)),(.5,(-1.6,-.35,.4)),(1,(0,0,0))])
                targets['l']+=p(0,-.02*math.sin(t*math.pi),-.03*math.sin(t*math.pi))
            if name=='bow_attack' and kind=='archer':
                targets['l']=eased_keys(t,[(0,grips['l']),(.2,p(-.22,-.18,-.62)),(.35,p(-.22,-.18,-.62)),(.43,p(-.22,-.19,-.58)),(1,grips['l'])])
                targets['r']=eased_keys(t,[(0,grips['r']),(.12,p(-.22,-.18,-.45)),(.29,p(-.22,-.18,-.18)),(.35,p(-.22,-.18,-.18)),(.41,p(-.17,-.16,-.12)),(.64,p(.30,-.27,-.17)),(.86,p(-.20,-.24,-.40)),(1,grips['r'])])
            if name=='idle':
                for side in targets:targets[side]+=p(0,.003*math.sin(t*math.tau),0)
            bpy.context.view_layer.update()
            for side in ['l','r']:
                fore,hand=bones['forearm_'+side],bones['hand_'+side]
                orient_bone(fore,elbows[side],targets[side])
                # Keep forearms joined to their hands over the small viewmodel arc.
                fore.scale.y=(targets[side]-elbows[side]).length/fore.bone.length
                bpy.context.view_layer.update()
                matrix=Euler(angles[side],'XYZ').to_matrix().to_4x4() @ hand.bone.matrix_local
                matrix.translation=targets[side];hand.matrix=matrix
                bpy.context.view_layer.update()
            nock=targets['l']+Vector((0,-.18*.95,0))
            if name=='bow_attack' and .1<=t<=.35:nock=targets['r'].copy()
            matrix=bones['string_nock'].bone.matrix_local.copy();matrix.translation=nock;bones['string_nock'].matrix=matrix
            bones['arrow_socket'].scale=(1,1,1) if kind=='archer' and (name=='idle' or name=='bow_attack' and t<=.35) else (.001,.001,.001)
            for b in bones:
                for path in ['rotation_quaternion','location','scale']:b.keyframe_insert(data_path=path,frame=frame,group=b.name)
        action=obj.animation_data.action;action.name=name;action.use_fake_user=True
        track=obj.animation_data.nla_tracks.new();track.name=name;strip=track.strips.new(name,1,action);strip.action_frame_end=61;track.mute=True
    obj.animation_data.action=None
    for b in bones:b.rotation_quaternion=(1,0,0,0);b.location=(0,0,0);b.scale=(1,1,1)
    bpy.context.scene.frame_set(1)
    save_export('fp_'+kind)

def arena():
    reset()
    # Flat playable corridor, with rocks and vegetation beyond its collision bounds.
    cube('Surrounding meadow',(0,18,-.83),(160,95,.35),8)
    cube('Limestone plinth',(0,0,-.48),(64,6,.9),10,bevel=.15)
    cube('Meadow top',(0,0,-.075),(64,6,.15),9,bevel=.02)
    cube('Worn march path',(0,0,.005),(57,3.65,.012),11)
    for i in range(100):
        x=RNG.uniform(-31.8,31.8);y=RNG.choice([-1,1])*RNG.uniform(3.2,4.8)
        ell('Limestone outcrop',(x,y,-.52),(RNG.uniform(.3,1.25),RNG.uniform(.35,.9),RNG.uniform(.4,1.05)),10,seg=6,rings=4,smooth=False)
    for i in range(150):
        x=RNG.uniform(-31.5,31.5);y=RNG.choice([-1,1])*RNG.uniform(2.8,3.5)
        for j in range(2):
            rod('Grass tuft',(x+j*.04,y,-.01),(x+j*.05+.03,y+.04,RNG.uniform(.14,.3)),.035,8,vertices=3,end_radius=0)
    config=json.loads((ROOT/'config/arena.json').read_text())
    for team,sgn,tile in [('blue',-1,13),('red',1,14)]:
        x=sgn*28
        cube('Guardian pedestal',(x,0,.19),(2.1,2.1,.38),10,bevel=.05)
        cube('Pedestal cap',(x,0,.42),(1.8,1.8,.12),11,bevel=.04)
        for sign in [-1,1]:
            cube('Carved guardian foot',(x+sign*.3,-.15,.63),(.4,.66,.3),10,bevel=.04)
            rod('Guardian leg',(x+sign*.3,0,.65),(x+sign*.3,0,1.65),.24,10,vertices=6)
        ell('Guardian cloak',(x,0,2.05),(.68,.38,.8),10,seg=8,rings=6,smooth=False)
        ell('Guardian head',(x,-.01,3.07),(.35,.31,.44),11,seg=8,rings=6,smooth=False)
        cube('Guardian brow',(x,-.28,3.17),(.65,.1,.12),10,bevel=.015)
        cube('Guardian nose',(x,-.33,3.02),(.12,.13,.23),10,bevel=.018)
        for sign in [-1,1]:
            rod('Guardian arm',(x+sign*.62,0,2.58),(x+sign*.35,-.42,1.95),.18,10,vertices=6)
        rod('Ceremonial sword',(x,-.46,.65),(x,-.46,2.17),.082,11,vertices=4)
        rod('Guardian sword hilt',(x-.31,-.46,1.86),(x+.31,-.46,1.86),.06,11,vertices=6)
        rod('Banner pole',(x,-2.4,.0),(x,-2.4,4.7),.06,2)
        cube('Hanging banner',(x+.52,-2.4,3.94),(1.0,.03,1.25),tile)
        cube('Banner trim',(x+.52,-2.423,3.35),(1.0,.02,.07),6)
        # Stylized original sun insignia.
        ell('Banner medallion',(x+.52,-2.43,4.03),(.18,.012,.18),7,seg=8,rings=5,smooth=False)
        mine=config['teams'][team]['mine'];mx,my=mine['x'],-mine['z']
        for i in range(7):
            dx=RNG.uniform(-.6,.6);dy=RNG.uniform(-.4,.4)
            ell('Gold-bearing stone',(mx+dx,my+dy,.25),(.4,.32,.4),10,seg=6,rings=4,smooth=False)
            ell('Gold vein',(mx+dx,my+dy-.08,.47),(.14,.12,.14),6,seg=5,rings=4,smooth=False)
        for anchor,position in config['teams'][team].items():
            empty=bpy.data.objects.new(team+'_'+anchor,None)
            empty.location=(position['x'],-position['z'],0)
            empty.empty_display_type='ARROWS';empty['arena_anchor']=True
            bpy.context.collection.objects.link(empty)
    # Restrained scenery forms a backdrop without occluding the playable strip.
    for i in range(18):
        x=(i-9)*5.5;y=RNG.uniform(10,21)
        rod('Cypress trunk',(x,y,-1),(x,y,2),.13,2,vertices=6)
        ell('Cypress crown',(x,y,2.1),(RNG.uniform(.55,.9),.65,RNG.uniform(1.8,3.0)),12,seg=7,rings=6,smooth=False)
    for i in range(18):
        ell('Distant ridge',((i-9)*9,35+RNG.random()*13,-1),(8,6,RNG.uniform(4,10)),8,seg=6,rings=4,smooth=False)
    for sign in [-1,1]:
        for i in range(3):
            cube('Ruined limestone column',(sign*(13+i*3),8+i,.5),(1.0,1.1,2+i*.35),10,bevel=.08)
        cube('Broken arch lintel',(sign*17,9,2.1),(5.5,1.1,.6),11,bevel=.09)
    join('Limestone ridge scenery')
    # Export anchor data directly from named authoring markers.
    for team in ['blue','red']:
        for name in ['statue','mine','spawn']:
            v=bpy.data.objects[team+'_'+name].location
            config['teams'][team][name]={'x':round(v.x,4),'z':round(-v.y,4)}
    (OUT/'arena.json').write_text(json.dumps(config,indent=2)+'\n')
    save_export('arena')

for role in ['miner','swordsman','archer']:
    character(role)
for role in ['swordsman','archer']:
    first_person(role)
arena()
print('AUTHORED', json.dumps(sorted(p.name for p in OUT.glob('*.glb'))))
