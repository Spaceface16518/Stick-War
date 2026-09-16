import json
import bpy
from .geometry import reset, cube, ell, rod, join, RNG, ROOT, OUT

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


