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
from mathutils import Vector

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
        ('weapon_socket',(-.43,-.04,.73),(-.43,-.15,.73),'hand_r'),
        ('bow_socket',(.43,-.04,.73),(.43,-.15,.73),'hand_l'),
        ('arrow_socket',(.43,-.20,.83),(.43,-.30,.83),'hand_l')
    ])
    for name, head, tail, parent in specs:
        bone = arm.edit_bones.new(name)
        bone.head, bone.tail = head, tail
        if parent:
            bone.parent = arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')
    obj.show_in_front = True
    return obj

def sword(center=(-.43,-.09,.77), bone='hand_r', scale=1, forward=1):
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

def bow(center=(.43,-.12,.78), bone='hand_l', scale=1, forward=1):
    c=Vector(center)
    points=[]
    for i in range(9):
        t=-1+i/4
        points.append(c+Vector((0,-.18*forward*(1-t*t),t*.52))*scale)
    for i in range(8):rod('Yew bow limb',points[i],points[i+1],.024*scale,2,bone,end_radius=.018*scale)
    rod('Linen string',points[0],points[-1],.005*scale,7,bone,vertices=4)
    rod('Leather bow grip',c+Vector((0,-.18*forward,-.07))*scale,c+Vector((0,-.18*forward,.07))*scale,.032*scale,3,bone)

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
        rod('Pickaxe handle',(-.43,-.06,.58),(-.43,-.32,1.42),.027,2,'hand_r')
        rod('Pickaxe head',(-.76,-.32,1.42),(-.1,-.32,1.42),.042,4,'hand_r',end_radius=.008)
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

def create_animations(obj,kind):
    bones=obj.pose.bones
    names=['idle','walk','carry','mine','melee_attack','bow_attack','hit','death']
    for name in names:
        duration={'idle':60,'walk':24,'carry':30,'mine':45,'melee_attack':24,'bow_attack':42,'hit':10,'death':28}[name]
        obj.animation_data_create();obj.animation_data.action=None
        for frame in range(1,duration+2,3):
            t=(frame-1)/duration;a=math.sin(t*math.tau)
            for b in bones:
                b.rotation_mode='XYZ';b.rotation_euler=(0,0,0);b.location=(0,0,0)
            bones['spine'].rotation_euler.x=.015*a
            if name in ('walk','carry'):
                for side,sign in [('l',1),('r',-1)]:
                    bones['thigh_'+side].rotation_euler.x=.48*a*sign
                    bones['shin_'+side].rotation_euler.x=max(0,-a*sign)*.5
                    bones['upper_arm_'+side].rotation_euler.x=-.24*a*sign
                bones['pelvis'].location.y=.018*abs(a)
                if name=='carry':bones['spine'].rotation_euler.x=.09
            if name=='mine':
                bones['upper_arm_r'].rotation_euler.x=-.7-.7*math.cos(t*math.tau)
                bones['forearm_r'].rotation_euler.x=-.35
                bones['spine'].rotation_euler.x=.16+.12*a
            if name=='melee_attack':
                swing=math.sin(t*math.pi)
                bones['upper_arm_r'].rotation_euler.x=-1.3*swing
                bones['upper_arm_r'].rotation_euler.z=-.8*swing
                bones['spine'].rotation_euler.y=.25*math.sin(t*math.tau)
            if name=='bow_attack':
                bones['upper_arm_l'].rotation_euler.x=-1.2
                bones['forearm_l'].rotation_euler.x=-.2
                bones['upper_arm_r'].rotation_euler.x=-1.15
                bones['upper_arm_r'].rotation_euler.z=-.8*math.sin(t*math.pi)
                bones['forearm_r'].rotation_euler.x=-1.0*math.sin(t*math.pi)
            if name=='hit':bones['spine'].rotation_euler.x=-.23*math.sin(t*math.pi)
            if name=='death':
                bones['root'].rotation_euler.x=min(1,t*1.5)*1.48
                bones['root'].location.z=-.14*min(1,t*1.5)
            for b in bones:
                b.keyframe_insert(data_path='rotation_euler',frame=frame,group=b.name)
                if b.name in ('root','pelvis'):b.keyframe_insert(data_path='location',frame=frame,group=b.name)
        action=obj.animation_data.action;action.name=name;action.use_fake_user=True
        track=obj.animation_data.nla_tracks.new();track.name=name
        strip=track.strips.new(name,1,action);strip.action_frame_start=1;strip.action_frame_end=duration+1
        track.mute=True
    obj.animation_data.action=None
    for b in bones:b.rotation_euler=(0,0,0);b.location=(0,0,0)
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
    # Author directly in camera coordinates, converted to Blender XYZ.
    def p(x,y,z):return (x,-z,y)
    for sign in [-1,1]:
        wrist=p(sign*.24,-.27,-.46)
        elbow=p(sign*.38,-.49,-.05)
        rod('Sleeve',elbow,p(sign*.30,-.35,-.28),.075,team=True,end_radius=.065)
        rod('Forearm',p(sign*.30,-.35,-.27),wrist,.059,1,end_radius=.044)
        rod('Bracer',p(sign*.27,-.30,-.38),wrist,.06,2,end_radius=.055)
        ell('Hand',wrist,(.06,.072,.05),1,seg=10,rings=6)
        for finger in range(3):
            ell('Knuckle',p(sign*.24+(finger-1)*.025,-.265,-.50),(.013,.021,.016),0,seg=6,rings=4)
    if kind=='swordsman':
        # Sword functions use Blender Z-up; camera grip below-right and blade above.
        sword(center=p(.26,-.28,-.58),bone=None,scale=.66,forward=-1)
    else:
        bow(center=p(-.24,-.22,-.53),bone=None,scale=.95,forward=-1)
        rod('Nocked arrow',p(-.22,-.22,-.4),p(-.22,-.22,-1.12),.008,2,vertices=6)
        ell('Arrowhead',p(-.22,-.22,-1.12),(.021,.04,.012),4,seg=6,rings=4)
    join('First person '+kind)
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
