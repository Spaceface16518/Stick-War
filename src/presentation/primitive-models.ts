import * as THREE from "three";
import type { Team, UnitKind } from "../simulation/types";
import { teamColors } from "./assets";
const templates = new Map<string, THREE.Group>();
export const material = (color: number) =>
  new THREE.MeshStandardMaterial({ color, roughness: 0.85 });
export function box(
  parent: THREE.Object3D,
  size: number[],
  position: number[],
  mat: THREE.Material,
): THREE.Mesh {
  const mesh = new THREE.Mesh(
    new THREE.BoxGeometry(size[0], size[1], size[2]),
    mat,
  );
  mesh.position.set(position[0], position[1], position[2]);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  parent.add(mesh);
  return mesh;
}
export function primitiveCharacter(kind: UnitKind, team: Team): THREE.Group {
  const key = kind + team;
  let template = templates.get(key);
  if (template) return template.clone(true);
  template = new THREE.Group();
  const cloth = material(teamColors[team]),
    skin = material(0xd2a27a),
    leather = material(0x56402d),
    steel = material(0x929994);
  steel.metalness = 0.4;
  box(template, [0.56, 0.65, 0.32], [0, 1.08, 0], cloth);
  box(template, [0.36, 0.36, 0.34], [0, 1.62, 0.02], skin);
  for (const side of [-1, 1]) {
    box(template, [0.19, 0.64, 0.22], [side * 0.17, 0.39, 0], leather);
    box(template, [0.21, 0.56, 0.21], [side * 0.4, 1.05, 0], cloth);
    box(template, [0.2, 0.22, 0.26], [side * 0.4, 0.75, 0.06], skin);
    box(template, [0.23, 0.16, 0.38], [side * 0.17, 0.09, 0.07], leather);
  }
  if (kind === "swordsman") {
    box(template, [0.43, 0.14, 0.4], [0, 1.8, 0], steel);
    box(template, [0.075, 0.85, 0.04], [-0.42, 1.07, 0.3], steel);
    box(template, [0.3, 0.05, 0.08], [-0.42, 0.69, 0.3], leather);
  }
  if (kind === "miner") {
    box(template, [0.07, 1, 0.07], [-0.43, 1.05, 0.27], leather);
    box(template, [0.65, 0.09, 0.08], [-0.43, 1.53, 0.27], steel);
    box(template, [0.4, 0.45, 0.27], [0, 0.93, -0.28], leather);
  }
  if (kind === "archer") {
    const bow = new THREE.Mesh(
      new THREE.TorusGeometry(0.46, 0.035, 5, 16, Math.PI),
      leather,
    );
    bow.position.set(-0.42, 1.12, 0.3);
    bow.rotation.z = -Math.PI / 2;
    template.add(bow);
    box(template, [0.2, 0.6, 0.2], [0.1, 1.22, -0.3], leather);
  }
  templates.set(key, template);
  return template.clone(true);
}
export function primitiveFirstPerson(kind: UnitKind, team: Team): THREE.Group {
  const key = "fp" + kind + team;
  let template = templates.get(key);
  if (template) return template.clone(true);
  template = new THREE.Group();
  const cloth = material(teamColors[team]),
    steel = material(0xb6c0b9),
    wood = material(0x695039);
  const hand = box(template, [0.13, 0.13, 0.38], [0.3, -0.32, -0.43], cloth);
  hand.rotation.x = -0.3;
  if (kind === "swordsman") {
    const blade = box(
      template,
      [0.075, 0.72, 0.025],
      [0.32, 0.07, -0.63],
      steel,
    );
    blade.rotation.x = -0.25;
    box(template, [0.26, 0.04, 0.07], [0.32, -0.28, -0.54], wood);
  } else {
    const bow = new THREE.Mesh(
      new THREE.TorusGeometry(0.42, 0.026, 5, 20, Math.PI),
      wood,
    );
    bow.rotation.z = Math.PI / 2;
    bow.position.set(-0.2, -0.15, -0.7);
    template.add(bow);
    box(template, [0.12, 0.12, 0.4], [-0.25, -0.3, -0.48], cloth);
  }
  templates.set(key, template);
  return template.clone(true);
}
