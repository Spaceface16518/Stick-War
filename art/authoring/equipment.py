import bpy
from mathutils import Vector
from .geometry import rod, ell, finish

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
    for i in range(8):
        limb = ('bow_lower' if i < 4 else 'bow_upper') if bone else None
        rod('Yew bow limb',points[i],points[i+1],.024*scale,2,limb,end_radius=.018*scale)
    if bone:
        nock=c+Vector((0,.18*forward,0))*scale
        string_segment(points[0],nock,'bow_lower','string_nock')
        string_segment(nock,points[-1],'string_nock','bow_upper')
        rod('Nocked arrow',nock,nock+Vector((0,-.85*forward,0)),.008,2,'arrow_socket',vertices=6)
        ell('Arrowhead',nock+Vector((0,-.86*forward,0)),(.025,.06,.015),4,'arrow_socket',seg=6,rings=4)
    else:
        rod('Linen string',points[0],points[-1],.005*scale,7,bone,vertices=4)
    rod('Leather bow grip',c+Vector((0,0,-.07))*scale,c+Vector((0,0,.07))*scale,.032*scale,3,bone)
