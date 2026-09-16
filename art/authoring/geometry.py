"""Original tabletop models. Run with Blender 5; no third-party Python packages.
Blender source: Z up, forward -Y. GLB: metres, Y up, forward +Z.
Art is deliberately reproducible, editable geometry, not a runtime character editor.
"""
import bpy
import math
import random
from pathlib import Path
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
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
