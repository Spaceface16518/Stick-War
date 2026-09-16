import * as THREE from "three";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";
import { MeshoptDecoder } from "three/addons/libs/meshopt_decoder.module.js";
import { clone } from "three/addons/utils/SkeletonUtils.js";
import { z } from "zod";
import type { Team } from "../simulation/types";
const schema = z.object({
  version: z.literal(1),
  assets: z.array(
    z.object({
      id: z.string(),
      file: z.string(),
      bytes: z.number().int().nonnegative(),
    }),
  ),
});
export const teamColors = { blue: 0x376e9e, red: 0xb95338 };
export class AssetStore {
  private loaded = new Map<string, GLTF>();
  private variants = new Map<string, THREE.Group>();
  async load(
    progress: (fraction: number, message: string) => void,
  ): Promise<void> {
    const base = new URL("assets/", document.baseURI);
    const response = await fetch(new URL("manifest.json", base));
    if (!response.ok)
      throw new Error(`Asset manifest: HTTP ${response.status}`);
    const manifest = schema.parse(await response.json());
    const total = manifest.assets.reduce((n, a) => n + a.bytes, 0);
    let received = 0;
    const loader = new GLTFLoader();
    loader.setMeshoptDecoder(MeshoptDecoder);
    for (const asset of manifest.assets) {
      const response = await fetch(new URL(asset.file, base));
      if (!response.ok) throw new Error(`${asset.id}: HTTP ${response.status}`);
      const reader = response.body?.getReader();
      let buffer: ArrayBuffer;
      if (reader) {
        const chunks: Uint8Array[] = [];
        let size = 0;
        while (true) {
          const part = await reader.read();
          if (part.done) break;
          chunks.push(part.value);
          size += part.value.length;
          received += part.value.length;
          progress(
            total ? Math.min(received / total, 1) : 0,
            `Unpacking ${asset.id.replaceAll("_", " ")}…`,
          );
        }
        const data = new Uint8Array(size);
        let offset = 0;
        for (const chunk of chunks) {
          data.set(chunk, offset);
          offset += chunk.length;
        }
        buffer = data.buffer;
      } else {
        buffer = await response.arrayBuffer();
        received += buffer.byteLength;
      }
      this.loaded.set(asset.id, await loader.parseAsync(buffer, base.href));
    }
    progress(1, "Battlefield ready");
  }
  has(key: string): boolean {
    return this.loaded.has(key);
  }
  instantiate(
    key: string,
    team: Team = "blue",
  ): { root: THREE.Group; clips: THREE.AnimationClip[] } | null {
    const asset = this.loaded.get(key);
    if (!asset) return null;
    const variantKey = `${key}:${team}`;
    let template = this.variants.get(variantKey);
    if (!template) {
      template = clone(asset.scene) as THREE.Group;
      template.traverse((object) => {
        if (object instanceof THREE.Mesh) {
          const materials = Array.isArray(object.material)
            ? object.material
            : [object.material];
          object.material = materials.map((material) => {
            if (!material.name.toLowerCase().includes("team")) return material;
            const copy = material.clone() as THREE.MeshStandardMaterial;
            copy.color.setHex(teamColors[team]);
            return copy;
          });
          if (object.material.length === 1)
            object.material = object.material[0];
        }
      });
      this.variants.set(variantKey, template);
    }
    return { root: clone(template) as THREE.Group, clips: asset.animations };
  }
}
