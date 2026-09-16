"""Geometry and skin weights only. Actions are authored in motion.py."""
import bpy
from mathutils import Vector
from .geometry import reset, ell, cube, rod, join
from .equipment import sword, bow
from .rig import rig

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
        ell('Boot heel '+side,(sign*.15,.015,.11),(.099,.11,.103),2,'foot_'+side)
        ell('Boot toe '+side,(sign*.15,-.14,.09),(.105,.13,.08),2,'toe_'+side)
        cube('Heel sole '+side,(sign*.15,.014,.028),(.202,.16,.045),3,'foot_'+side,bevel=.012)
        cube('Forefoot sole '+side,(sign*.15,-.153,.028),(.207,.18,.045),3,'toe_'+side,bevel=.012)
        for level in range(3):
            z=.2+level*.065
            rod('Boot cross lace', (sign*.15-.055,-.083,z), (sign*.15+.055,-.083,z+.03), .007,6,'shin_'+side,vertices=5)
            rod('Boot cross lace', (sign*.15+.055,-.088,z), (sign*.15-.055,-.088,z+.03), .007,6,'shin_'+side,vertices=5)
        for finger in range(4):
            x=sign*.43+(finger-1.5)*.022
            ell('Curled finger '+side,(x,-.093,.745),(.016,.025,.041),1,'hand_'+side,seg=6,rings=5)
            ell('Knuckle '+side,(x,-.085,.78),(.017,.022,.02),0,'hand_'+side,seg=6,rings=4)
        for height in [.832,.896]:
            rod('Bracer edge '+side,(sign*.43-.047,-.087,height),(sign*.43+.047,-.087,height),.008,6,'forearm_'+side,vertices=5)
        rod('Tunic shoulder seam',(sign*.09,-.16,1.37),(sign*.27,-.12,1.33),.009,7,'spine',vertices=5)
        ell('Cheek plane',(sign*.09,-.102,1.60),(.051,.035,.04),0,'head',seg=8,rings=5)
        rod('Eyelid',(sign*.032,-.134,1.703),(sign*.084,-.122,1.70),.006,0,'head',vertices=5)
    rod('Lower lip',(-.042,-.143,1.566),(.042,-.143,1.566),.012,0,'head',vertices=6)
    rod('Collar edge',(-.16,-.12,1.41),(0,-.18,1.34),.018,7,'spine',vertices=6)
    rod('Collar edge',(0,-.18,1.34),(.16,-.12,1.41),.018,7,'spine',vertices=6)
    for x in [-.18,-.09,.09,.18]:
        rod('Tunic pleat',(x,-.157,.84),(x*.9,-.173,.98),.008,13,'pelvis',vertices=5)
    cube('Belt pouch',(.25,-.16,.95),(.14,.09,.15),2,'pelvis',bevel=.025)
    cube('Pouch flap',(.25,-.211,.987),(.14,.018,.058),0,'pelvis',bevel=.012)
    if kind=='swordsman':
        ell('Open steel helmet',(0,.01,1.76),(.17,.146,.106),4,'head',seg=12,rings=7)
        for sign in [-1,1]:
            cube('Helmet cheek guard',(sign*.139,.012,1.64),(.032,.14,.17),4,'head',bevel=.014)
            ell('Steel shoulder',(sign*.325,.012,1.37),(.135,.15,.084),4,'upper_arm_'+('l' if sign==1 else 'r'))
        rod('Helmet brow rim',(-.145,-.09,1.746),(.145,-.09,1.746),.024,5,'head')
        cube('Chest leather harness',(0,-.161,1.235),(.057,.018,.36),2,'spine')
        ell('Fitted breastplate',(0,-.05,1.235),(.26,.138,.21),4,'spine',seg=12,rings=7)
        rod('Breastplate rolled rim',(-.205,-.13,1.35),(.205,-.13,1.35),.018,5,'spine')
        for sign in [-1,1]:
            for level in range(2):
                ell('Overlapping shoulder lames',(sign*(.345+level*.025),.012,1.31-level*.048),(.119,.125,.048),4,'upper_arm_'+('l' if sign==1 else 'r'),seg=8,rings=5)
            for height in [1.16,1.28]:
                ell('Armor rivet',(sign*.18,-.155,height),(.013,.012,.013),6,'spine',seg=6,rings=4)
        rod('Helmet crest ridge',(0,-.12,1.80),(0,.12,1.82),.022,5,'head')
        for sign in [-1,1]:
            rod('Chin strap',(sign*.135,-.03,1.68),(sign*.08,-.065,1.49),.012,2,'head',vertices=6)
        sword()
    elif kind=='miner':
        ell('Wool cap',(0,.016,1.77),(.157,.142,.105),2,'head')
        ell('Beard',(0,-.10,1.53),(.10,.064,.085),2,'head')
        cube('Leather apron',(0,-.166,1.02),(.37,.028,.47),2,'spine',bevel=.02)
        ell('Canvas ore sack',(0,.23,1.14),(.21,.145,.27),6,'spine')
        rod('Sack strap',(-.17,-.15,1.38),(.15,-.18,1.02),.025,3,'spine')
        rod('Pickaxe handle',(-.43,-.045,.54),(-.43,-.045,1.42),.027,2,'weapon_socket')
        rod('Pickaxe head',(-.76,-.045,1.42),(-.1,-.045,1.42),.042,4,'weapon_socket',end_radius=.008)
        for x in [-.14,.14]:
            rod('Apron stitching',(x,-.184,.82),(x,-.184,1.2),.006,7,'spine',vertices=5)
        cube('Apron pocket',(0,-.191,.96),(.2,.024,.14),0,'spine',bevel=.014)
        rod('Sack drawstring',(-.12,.23,1.38),(.12,.23,1.38),.018,7,'spine')
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
        for sign in [-1,1]:
            rod('Hood stitched opening',(sign*.105,-.112,1.80),(sign*.143,-.091,1.55),.01,7,'head',vertices=6)
        rod('Quiver brass rim',(.11,.22,1.5),(.27,.22,1.5),.015,6,'spine')
        cube('Leather chest guard',(-.10,-.172,1.20),(.20,.022,.22),2,'spine',bevel=.015)
        bow()
    mesh=join(kind.title()+' painted model')
    skeleton=rig()
    mesh.parent=skeleton
    mod=mesh.modifiers.new('Humanoid skin','ARMATURE');mod.object=skeleton
    return skeleton

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
        for finger in range(4):
            at=wrist+Vector(((finger-1.5)*.024,.035,0))
            ell('Knuckle',at,(.015,.023,.017),0,'hand_'+side,seg=8,rings=5)
            ell('Curled finger',at+Vector((0,.012,-.027)),(.014,.022,.034),1,'hand_'+side,seg=8,rings=5)
        ell('Thumb',wrist+Vector((-sign*.056,.02,.008)),(.025,.043,.024),1,'hand_'+side,seg=8,rings=5)
        for along in [.72,.87]:
            at=elbow.lerp(wrist,along)
            rod('Bracer seam',at+Vector((-.049,0,.046)),at+Vector((.049,0,.046)),.006,6,'forearm_'+side,vertices=5)
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
    specs += [('weapon_socket',grips['r'],grips['r']+Vector((0,.1,0)),'hand_r'),('bow_socket',grips['l'],grips['l']+Vector((0,.1,0)),'hand_l'),('string_nock',nock,nock+Vector((0,0,.1)),'bow_socket'),('arrow_socket',nock,nock+Vector((0,.1,0)),'string_nock')]
    specs += [('bow_upper',grips['l'],grips['l']+Vector((0,-.171,.494)),'bow_socket'),('bow_lower',grips['l'],grips['l']+Vector((0,-.171,-.494)),'bow_socket')]
    for name,head,tail,parent in specs:
        b=arm.edit_bones.new(name);b.head=head;b.tail=tail
        if parent:b.parent=arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode='OBJECT')
    mesh.parent=obj;mod=mesh.modifiers.new('First person skin','ARMATURE');mod.object=obj
    return obj
