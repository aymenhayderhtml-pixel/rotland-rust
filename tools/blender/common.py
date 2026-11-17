"""Shared helpers for every generated .glb model. Run models through `run_model`.

Geometry is built in bmesh. Each face stores a palette color index, and the export
writes UVs that point at that color's cell in assets/textures/palette.png.
"""
import math
import os
import random

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

import palette

LAYER = "pal"
MATERIAL_NAME = "palette"
MATERIAL_NAME_B = "palette_b"  # second sheet; only exported when a model uses a sheet B color
PREVIEW_SIZE = 384
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
MODELS_DIR = os.path.join(ROOT, "assets", "models")
PREVIEW_DIR = os.path.join(MODELS_DIR, "_previews")


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def palette_material(sheet="A"):
    """Material for palette sheet A (`palette`) or sheet B (`palette_b`). Same settings on both."""
    name, path = (MATERIAL_NAME, palette.PNG_PATH) if sheet == "A" else (MATERIAL_NAME_B, palette.PNG_PATH_B)
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()
    image = bpy.data.images.load(path, check_existing=True)
    image.pack()  # embed in the .glb so no loose palette copies are exported
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = image
    tex.interpolation = "Closest"  # hard cell edges, no blur between colors
    bsdf = nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Roughness"].default_value = 0.9
    bsdf.inputs["Metallic"].default_value = 0.0
    out = nodes.new("ShaderNodeOutputMaterial")
    links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return mat


def new_bm():
    bm = bmesh.new()
    bm.faces.layers.int.new(LAYER)
    return bm


def paint(bm, color_name):
    paint_faces(bm, list(bm.faces), color_name)


def paint_faces(bm, faces, color_name):
    layer = bm.faces.layers.int[LAYER]
    key = palette.key_of(color_name)
    for face in faces:
        face[layer] = key


def transform(bm, loc=(0.0, 0.0, 0.0), rot=(0.0, 0.0, 0.0), scale=(1.0, 1.0, 1.0)):
    """Scale, then rotate (Euler XYZ, radians), then move."""
    matrix = (Matrix.Translation(loc)
              @ Euler(rot, "XYZ").to_matrix().to_4x4()
              @ Matrix.Diagonal((scale[0], scale[1], scale[2], 1.0)))
    bmesh.ops.transform(bm, matrix=matrix, verts=list(bm.verts))
    bm.normal_update()
    return bm


def make_frustum(r_bottom, r_top, z0, z1, segments=8, center=(0.0, 0.0), rot_z=0.0, color="bark_dark"):
    """Cylinder (equal radii), cone (one radius 0) or truncated cone, standing from z0 to z1."""
    bm = new_bm()
    cx, cy = center

    def ring(radius, z):
        return [bm.verts.new((cx + math.cos(a) * radius, cy + math.sin(a) * radius, z))
                for a in (rot_z + 2 * math.pi * i / segments for i in range(segments))]

    bottom = ring(r_bottom, z0) if r_bottom > 0 else None
    top = ring(r_top, z1) if r_top > 0 else None
    apex_bottom = None if bottom else bm.verts.new((cx, cy, z0))
    apex_top = None if top else bm.verts.new((cx, cy, z1))

    for i in range(segments):
        j = (i + 1) % segments
        corners = [bottom[i] if bottom else apex_bottom,
                   bottom[j] if bottom else apex_bottom,
                   top[j] if top else apex_top,
                   top[i] if top else apex_top]
        # Drop repeated apex corners so cones get triangles, not degenerate quads
        ring_verts = []
        for v in corners:
            if not ring_verts or ring_verts[-1] is not v:
                ring_verts.append(v)
        if len(ring_verts) > 1 and ring_verts[0] is ring_verts[-1]:
            ring_verts.pop()
        if len(ring_verts) >= 3:
            bm.faces.new(ring_verts)
    if bottom:
        bm.faces.new(list(reversed(bottom)))
    if top:
        bm.faces.new(top)
    paint(bm, color)
    return bm


def make_icosphere(subdiv=2, jitter=0.0, seed=0, radius=(1.0, 1.0, 1.0), loc=(0.0, 0.0, 0.0),
                   rot=(0.0, 0.0, 0.0), color="rock_mid"):
    """Icosphere with random radial vertex noise. `radius` stretches it per axis."""
    bm = new_bm()
    bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=1.0)
    rng = random.Random(seed)
    for v in bm.verts:
        v.co *= 1.0 + jitter * rng.uniform(-1.0, 1.0)
    transform(bm, loc=loc, rot=rot, scale=radius)
    paint(bm, color)
    return bm


def make_box(size, loc=(0.0, 0.0, 0.0), color="wood_crate"):
    bm = new_bm()
    bmesh.ops.create_cube(bm, size=1.0)
    transform(bm, loc=loc, scale=size)
    paint(bm, color)
    return bm


def add_quad(bm, corners, outward, color):
    """Adds one quad, flipped if needed so its normal points along `outward`."""
    verts = [bm.verts.new(c) for c in corners]
    normal = (Vector(corners[1]) - Vector(corners[0])).cross(Vector(corners[2]) - Vector(corners[0]))
    if normal.dot(Vector(outward)) < 0:
        verts.reverse()
    face = bm.faces.new(verts)
    paint_faces(bm, [face], color)


def combine(bms):
    """Merges several bmeshes into one, keeping each face's color."""
    out = new_bm()
    for src in bms:
        vmap = {}
        for v in src.verts:
            vmap[v] = out.verts.new(v.co)
        out.verts.ensure_lookup_table()
        src_layer = src.faces.layers.int[LAYER]
        dst_layer = out.faces.layers.int[LAYER]
        for face in src.faces:
            new_face = out.faces.new([vmap[v] for v in face.verts])
            new_face[dst_layer] = face[src_layer]
        src.free()
    return out


MARKER_COLORS = {"loot": "brass", "spawn": "red_paint", "door": "water_blue",
                 "npc": "soda_purple", "player": "leaf_light", "gate": "metal", "campfire": "red_paint"}


class Model:
    """One exported .glb: visual parts merge into one mesh, colliders stay as separate objects.

    Props use the default origin (bottom center). Weapons pass `grip`, the point in the
    build coordinates that becomes the origin (the hand position).
    `top_preview=True` also renders `<name>_top.png` with roof parts left out, so the inside shows.
    Marker empties (loot_, spawn_, door_) get colored balls in the previews only, never exported.
    """

    def __init__(self, name, category, budget, grip=None, top_preview=False, view=None):
        self.name = name
        self.category = category
        self.budget = budget
        self.grip = grip
        self.top_preview = top_preview
        self.view = view  # preview camera direction (buildings look at their front, +Y)
        self._visual = []  # (bmesh, is_roof)
        self._colliders = []
        self._parts = []  # (label, bmesh): named separate meshes (a gun's magazine), exported as their own nodes
        self._empties = []  # (label, loc, scale, rot_z)

    def add(self, bm, roof=False):
        """Visual part. `roof=True` parts are left out of the top-down preview only."""
        self._visual.append((bm, roof))

    def add_collider(self, label, bm):
        """Named `<model>_<label>-convcolonly`, so Godot makes a convex collision shape from it."""
        self._colliders.append((label, bm))

    def add_part(self, label, bm):
        """Visual part exported as its own node named `label` (e.g. `mag`), so code can hide or move it."""
        self._parts.append((label, bm))

    def add_empty(self, label, loc, scale=(1.0, 1.0, 1.0), rot_z=0.0):
        """Named empty (e.g. `grip_L`) exported as a Godot Node3D at `loc` (build coordinates).
        `scale` is kept on the node, so `lot_size` can carry the footprint in meters.
        `rot_z` (radians) turns it: +Y is forward, so a marker facing -X uses +pi/2."""
        self._empties.append((label, loc, scale, rot_z))

    def finalize_and_export(self):
        """Applies transforms, sets origin (bottom center, or the grip), exports, renders previews. Returns (tris, ok)."""
        # Copies for the top view are taken first: combine() frees the originals
        top_parts = [bm.copy() for bm, roof in self._visual if not roof] if self.top_preview else []
        visual = combine([bm for bm, _ in self._visual])
        colliders = [(label, bm) for label, bm in self._colliders]
        parts = [(label, bm) for label, bm in self._parts]

        if self.grip is not None:
            offset = (-self.grip[0], -self.grip[1], -self.grip[2])
        else:
            xs = [v.co.x for v in visual.verts]
            ys = [v.co.y for v in visual.verts]
            zs = [v.co.z for v in visual.verts]
            offset = (-(min(xs) + max(xs)) / 2, -(min(ys) + max(ys)) / 2, -min(zs))
        for bm in [visual] + [bm for _, bm in colliders] + [bm for _, bm in parts] + top_parts:
            bmesh.ops.translate(bm, vec=offset, verts=list(bm.verts))
            bmesh.ops.triangulate(bm, faces=list(bm.faces))
            bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))

        tris = len(visual.faces) + sum(len(bm.faces) for _, bm in parts)
        ok = tris <= self.budget
        obj = _to_object(visual, self.name, painted=True)
        collider_objs = [_to_object(bm, f"{self.name}_{label}-convcolonly", painted=False)
                         for label, bm in colliders]
        for label, bm in parts:
            _to_object(bm, label, painted=True)
        for label, loc, scale, rot_z in self._empties:
            empty = bpy.data.objects.new(label, None)
            empty.empty_display_type = "PLAIN_AXES"
            empty.empty_display_size = 0.05
            empty.location = (loc[0] + offset[0], loc[1] + offset[1], loc[2] + offset[2])
            empty.rotation_euler = (0.0, 0.0, rot_z)
            empty.scale = scale
            bpy.context.scene.collection.objects.link(empty)

        out_path = os.path.join(MODELS_DIR, self.category, self.name + ".glb")
        _export_glb(out_path)
        # Colliders can sit exactly on the visible faces (the crate box), so hide them for the preview
        for collider in collider_objs:
            collider.hide_render = True
        obj["_is_weapon"] = self.grip is not None  # read by the preview framing
        markers = self._marker_objects(offset)
        render_preview(self.name, obj, view=self.view)
        _remove_objects(markers)
        if self.top_preview:
            obj.hide_render = True  # the roof is in `obj`, so hide the whole visual for the top view
            top_obj = _to_object(combine(top_parts), self.name + "_topview", painted=True)
            markers = self._marker_objects(offset)
            render_preview(self.name, top_obj, top=True)
            _remove_objects(markers + [top_obj])
        verdict = "OK" if ok else "OVER BUDGET"
        print(f"MODEL {self.name}: {tris} tris (budget {self.budget}) {verdict} -> {out_path}")
        return tris, ok

    def _marker_objects(self, offset):
        """Preview-only colored balls at the marker empties. Not exported."""
        objs = []
        for label, loc, _, _ in self._empties:
            kind = label.split("_")[0]
            if kind not in MARKER_COLORS:
                continue
            at = (loc[0] + offset[0], loc[1] + offset[1], loc[2] + offset[2])
            bm = make_icosphere(subdiv=1, radius=(0.14, 0.14, 0.14), loc=at, color=MARKER_COLORS[kind])
            objs.append(_to_object(bm, "marker_" + label, painted=True))
        return objs


def _remove_objects(objs):
    for obj in objs:
        bpy.data.objects.remove(obj, do_unlink=True)


def _to_object(bm, name, painted):
    layer = bm.faces.layers.int.get(LAYER)
    keys = [face[layer] for face in bm.faces] if painted else []  # read before bm is freed
    if painted:
        uv = bm.loops.layers.uv.new("UVMap")
        for face in bm.faces:
            u, v = palette.uv_center(palette.name_of_key(face[layer]))
            for loop in face.loops:
                loop[uv].uv = (u, v)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = False  # flat shading, the low-poly look
    if painted:
        # Sheet A is always slot 0, so sheet-A-only models export exactly as before (one material)
        sheets = ["A"] + (["B"] if any(palette.sheet_of_key(k) == "B" for k in keys) else [])
        for sheet in sheets:
            mesh.materials.append(palette_material(sheet))
        for poly, key in zip(mesh.polygons, keys):
            poly.material_index = sheets.index(palette.sheet_of_key(key))
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def _export_glb(path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for obj in bpy.context.scene.objects:
        obj.select_set(True)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True,
                              export_yup=True, export_apply=True)


def render_preview(name, obj, top=False, view=None):
    """384x384 PNG: 3/4 view (`name.png`) or straight down (`name_top.png`). Workbench first, Cycles CPU fallback."""
    os.makedirs(PREVIEW_DIR, exist_ok=True)
    path = os.path.join(PREVIEW_DIR, name + ("_top" if top else "") + ".png")
    scene = bpy.context.scene
    _add_camera_and_sun(scene, obj, top=top, view=view)
    scene.render.resolution_x = PREVIEW_SIZE
    scene.render.resolution_y = PREVIEW_SIZE
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.view_settings.view_transform = "Standard"  # keep palette colors exactly as written
    scene.render.filepath = path

    engine = None
    if os.path.exists(path):
        os.remove(path)
    try:
        _setup_workbench(scene)
        bpy.ops.render.render(write_still=True)
        if not os.path.exists(path):
            raise RuntimeError("Workbench wrote no file")
        engine = "BLENDER_WORKBENCH"
    except Exception as err:
        print(f"preview: Workbench failed ({err}), falling back to Cycles CPU")
        if os.path.exists(path):
            os.remove(path)
        _setup_cycles(scene)
        bpy.ops.render.render(write_still=True)
        if not os.path.exists(path):
            raise RuntimeError(f"preview for {name} was not written")
        engine = "CYCLES"
    print(f"preview {name}: engine {engine} -> {path}")


def _gray_world(scene):
    world = bpy.data.worlds.new("PreviewWorld")
    world.use_nodes = False
    world.color = (0.78, 0.78, 0.78)
    scene.world = world


def _setup_workbench(scene):
    scene.render.engine = "BLENDER_WORKBENCH"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "TEXTURE"  # shows the palette texture
    shading.background_type = "WORLD"
    scene.display.render_aa = "8"
    _gray_world(scene)


def _setup_cycles(scene):
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 16
    _gray_world(scene)


def _add_camera_and_sun(scene, obj, top=False, view=None):
    for old in ("PreviewCamera", "PreviewSun"):  # a second render (top view) replaces the first camera and sun
        if old in bpy.data.objects:
            bpy.data.objects.remove(bpy.data.objects[old], do_unlink=True)
    corners = [obj.matrix_world @ Vector(c) for c in obj.bound_box]
    lo = Vector((min(c.x for c in corners), min(c.y for c in corners), min(c.z for c in corners)))
    hi = Vector((max(c.x for c in corners), max(c.y for c in corners), max(c.z for c in corners)))
    center = (lo + hi) / 2
    # Weapons are small and thin, so they get a smaller frame floor than props (0.5 m)
    floor = 0.12 if getattr(obj, "_is_weapon", False) else 0.5
    radius = max((hi - lo).length / 2, floor)

    cam_data = bpy.data.cameras.new("PreviewCamera")
    cam_data.angle = math.radians(40)
    if top:
        # Orthographic from above: posts and walls stay straight, with no perspective slant
        cam_data.type = "ORTHO"
        cam_data.ortho_scale = 2.2 * radius
    cam = bpy.data.objects.new("PreviewCamera", cam_data)
    scene.collection.objects.link(cam)
    direction = Vector(view or (1.0, -1.0, 0.7)).normalized()  # 3/4 view from the front right, above
    if top:
        direction = Vector((0.0, -0.02, 1.0)).normalized()  # straight down; the tiny tilt keeps the camera upright
    distance = radius / math.sin(cam_data.angle / 2) * 1.1
    if top:
        distance = radius * 3.0
    cam.location = center + direction * distance
    cam.rotation_euler = (center - cam.location).to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam

    sun_data = bpy.data.lights.new("PreviewSun", "SUN")
    sun_data.energy = 3.0
    sun = bpy.data.objects.new("PreviewSun", sun_data)
    scene.collection.objects.link(sun)
    sun.rotation_euler = (math.radians(50), 0.0, math.radians(-30))


def object_from_bm(bm, name):
    """Public helper for icon renders: a palette-textured object from a bmesh (not exported)."""
    return _to_object(bm, name, painted=True)


def run_model(name, category, budget, build_fn, grip=None, top_preview=False, view=None):
    """Clean scene, build the model with build_fn(model), export and preview. Returns True if within budget."""
    reset_scene()
    model = Model(name, category, budget, grip=grip, top_preview=top_preview, view=view)
    build_fn(model)
    _, ok = model.finalize_and_export()
    return ok
