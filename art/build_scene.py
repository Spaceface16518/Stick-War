"""Export saved Blender sources; explicitly opt into geometry/action regeneration."""
import argparse
import hashlib
import json
import sys
from pathlib import Path
import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from authoring.geometry import SRC, OUT, ROOT
from authoring.characters import character, first_person
from authoring.arena import arena
from authoring.motion import create_animations

parser = argparse.ArgumentParser()
parser.add_argument('--author', action='store_true', help='Regenerate all model geometry and rig actions')
parser.add_argument('--animate', action='store_true', help='Regenerate rig actions in existing sources; preserve meshes')
parser.add_argument('--asset', choices=['miner', 'swordsman', 'archer', 'fp_swordsman', 'fp_archer', 'arena'])
options = parser.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else [])


def export(name):
    bpy.ops.export_scene.gltf(filepath=str(OUT/(name+'.glb')), export_format='GLB',
        export_yup=True, export_animations=True, export_animation_mode='ACTIONS',
        export_anim_slide_to_zero=True, export_force_sampling=False,
        export_skins=True, export_morph=False, export_cameras=False, export_lights=False,
        export_extras=True, export_apply=False)


def mesh_signature():
    """Detect accidental changes to geometry, topology, materials or skin weights."""
    meshes = []
    for obj in bpy.context.scene.objects:
        if obj.type != 'MESH':
            continue
        meshes.append((obj.name, [tuple(v.co) for v in obj.data.vertices],
            [tuple(p.vertices) for p in obj.data.polygons],
            [(g.name, g.index) for g in obj.vertex_groups],
            [[(g.group, g.weight) for g in v.groups] for v in obj.data.vertices],
            [m.name for m in obj.data.materials]))
    return hashlib.sha256(json.dumps(meshes).encode()).hexdigest()


assets = [options.asset] if options.asset else ['miner', 'swordsman', 'archer', 'fp_swordsman', 'fp_archer', 'arena']
for name in assets:
    fp = name.startswith('fp_')
    kind = name.removeprefix('fp_')
    if options.author:
        if name == 'arena':
            arena()
        else:
            obj = first_person(kind) if fp else character(kind)
            create_animations(obj, kind, fp)
    else:
        bpy.ops.wm.open_mainfile(filepath=str(SRC/(name+'.blend')))
        if options.animate and name != 'arena':
            before = mesh_signature()
            obj = next(o for o in bpy.context.scene.objects if o.type == 'ARMATURE')
            create_animations(obj, kind, fp)
            assert mesh_signature() == before, 'Animation regeneration changed a model mesh'
            print('PRESERVED GEOMETRY', name, before)
    if options.author or options.animate and name != 'arena':
        bpy.context.scene['asset_id'] = name
        bpy.context.scene['convention'] = 'Metres, feet at origin, glTF +Z forward; named shared rig'
        bpy.ops.wm.save_as_mainfile(filepath=str(SRC/(name+'.blend')))
    export(name)
    if name != 'arena' and not fp:
        for mesh in [o for o in bpy.context.scene.objects if o.type == 'MESH']:
            bpy.context.view_layer.objects.active = mesh
            triangles = sum(len(p.vertices)-2 for p in mesh.data.polygons)
            dec = mesh.modifiers.new('Commander geometry reduction', 'DECIMATE')
            dec.ratio = min(.40, 1800/max(1, triangles))
            bpy.ops.object.modifier_move_up(modifier=dec.name)
            bpy.ops.object.modifier_apply(modifier=dec.name)
            mesh.data.validate()
        export(name+'_lod')
    if name == 'arena':
        config = json.loads((ROOT/'config/arena.json').read_text())
        for team in ['blue', 'red']:
            for anchor in ['statue', 'mine', 'spawn']:
                p = bpy.data.objects[team+'_'+anchor].location
                config['teams'][team][anchor] = {'x': round(p.x, 4), 'z': round(-p.y, 4)}
        (OUT/'arena.json').write_text(json.dumps(config, indent=2)+'\n')
print('EXPORTED', assets)
