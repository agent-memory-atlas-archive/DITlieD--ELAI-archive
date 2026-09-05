Source: Blender CLI (headless) + bpy Python API + FFmpeg + FFprobe
Reliability: HIGH (official Blender docs.blender.org, official FFmpeg.org)
Files examined: docs.blender.org/manual/en/latest/advanced/command_line/arguments.html, docs.blender.org/api/current/bpy.ops.import_scene.html, docs.blender.org/api/current/bmesh.html, docs.blender.org/api/current/bpy.types.Mesh.html, ffmpeg.org/ffprobe.html, ayosec.github.io/ffmpeg-filters-docs/8.0/Filters/Audio/silencedetect.html, ayosec.github.io/ffmpeg-filters-docs/8.0/Filters/Audio/astats.html
Verified: 2026-05-11

=== SECTION 1: BLENDER CLI ===

CLI Invocation format:
  blender [args...] [file.blend] [args...]
  blender --background --python script.py -- arg1 arg2
  blender -b --python script.py --python-exit-code 1 -- arg1 arg2

CRITICAL: Arguments are executed in the order they are given. Order matters.
  Correct:  blender -b file.blend --python script.py
  Correct:  blender -b --python script.py (no blend file, script creates scene)
  Wrong:    blender --python script.py -b  (script may run with GUI briefly)

Key CLI flags:
  -b | --background          Run without GUI. Audio disabled. No display required.
  -P | --python <filepath>   Execute Python script file inside Blender.
  --python-expr <expr>       Execute single Python expression.
  --python-exit-code <N>     Exit with code N (range 0-255) if Python exception raised. 0=disable.
  --python-use-system-env    Allow PYTHONPATH and user site-packages (off by default).
  -y | --enable-autoexec     Enable auto Python execution (off by default for security).
  --                         End of Blender options. Everything after goes to sys.argv in script.
  -o | --render-output <path>  Set render output path. Use // prefix for blend-relative.
  -F | --render-format <fmt>   Set format: TGA RAWTGA JPEG IRIS PNG BMP HDR TIFF OPEN_EXR FFMPEG.
  -x | --use-extension <bool>  Add format extension to filename (1=yes, 0=no).

Argument parsing in script (accessing -- args):
  import sys
  args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []

Exit code conventions (Blender process):
  0   Normal exit (sys.exit(0) or script completes without error)
  1   Python exception raised (when --python-exit-code not set, Blender may still exit 0 — set --python-exit-code 1 to force)
  N   Whatever sys.exit(N) is called in the script
  139 Segfault (SIGSEGV) — malformed geometry, corrupt file, driver crash
  137 SIGKILL — OOM killer terminated process (Linux), timeout kill
  Note: Blender does NOT emit structured exit codes for specific error types beyond Python exceptions. Use --python-exit-code 1 and sys.exit(1) in script for all error cases.

Subprocess invocation from Rust (std::process::Command pattern):
  Command::new("blender")
    .args(["-b", "--python", script_path, "--python-exit-code", "1", "--", filepath, output_dir])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()

Timeout handling from Rust: use tokio::time::timeout wrapping child.wait(). On timeout, call child.kill() which sends SIGKILL. Exit status will be signal-terminated (not a clean exit code on Unix).

=== SECTION 2: BPY IMPORT OPERATORS ===

CRITICAL VERSION BREAK at Blender 4.0:
  bpy.ops.import_scene.obj  -> REMOVED in Blender 4.0. Replace with bpy.ops.wm.obj_import.
  bpy.ops.export_scene.obj  -> REMOVED in Blender 4.0. Replace with bpy.ops.wm.obj_export.

CRITICAL VERSION BREAK at Blender 4.5 / 5.0:
  bpy.ops.import_scene.fbx  -> LEGACY (Python-based). Still works but marked deprecated.
                               New C++ importer: bpy.ops.wm.fbx_import (Blender 4.5+, default in 5.0).

glTF status (Blender 4.x):
  bpy.ops.import_scene.gltf -> STILL VALID in Blender 4.x. Not removed. Addon-based (KhronosGroup glTF-Blender-IO, bundled).

OBJ import (Blender 4.0+):
  bpy.ops.wm.obj_import(
    filepath="path/to/file.obj",   # mandatory, str
    global_scale=1.0,              # optional, float, default 1.0
    clamp_size=0.0,                # optional, float, 0=disabled
    forward_axis='NEGATIVE_Z',     # optional, str
    up_axis='Y',                   # optional, str
    use_split_objects=True,        # optional, bool
    use_split_groups=False,        # optional, bool
    import_vertex_groups=False,    # optional, bool
  )
  Returns: {'FINISHED'} on success, {'CANCELLED'} on failure.
  Adds imported objects to bpy.context.selected_objects.

glTF import (Blender 3.x, 4.x — all versions):
  bpy.ops.import_scene.gltf(
    filepath="path/to/file.glb",   # mandatory, str (.glb or .gltf)
    filter_glob="*.glb;*.gltf",    # optional
    loglevel=0,                    # optional, int
    import_pack_images=True,       # optional, bool
    merge_vertices=False,          # optional, bool
    import_shading='NORMALS',      # optional, str
  )
  Returns: {'FINISHED'} on success, {'CANCELLED'} on failure.

FBX import — use version-aware pattern:
  # For Blender < 4.5 (legacy Python importer):
  bpy.ops.import_scene.fbx(
    filepath="path/to/file.fbx",   # mandatory, str
    global_scale=1.0,              # optional, float
    use_anim=True,                 # optional, bool
    ignore_leaf_bones=False,       # optional, bool
    use_custom_normals=True,       # optional, bool
    axis_forward='-Z',             # optional, str
    axis_up='Y',                   # optional, str
  )
  # For Blender 4.5+ (new C++ importer bpy.ops.wm.fbx_import):
  bpy.ops.wm.fbx_import(
    filepath="path/to/file.fbx",   # mandatory, str
    global_scale=1.0,              # optional, float
    use_anim=True,                 # optional, bool
    use_custom_normals=True,       # optional, bool
    use_custom_props=True,         # optional, bool
    ignore_leaf_bones=False,       # optional, bool
    validate_meshes=False,         # optional, bool
    import_subdivision=False,      # optional, bool
  )

Version detection pattern in script:
  import bpy
  ver = bpy.app.version  # tuple (major, minor, patch)
  if ver >= (4, 5, 0):
      bpy.ops.wm.fbx_import(filepath=path)
  else:
      bpy.ops.import_scene.fbx(filepath=path)

Post-import: get newly imported objects:
  imported = [o for o in bpy.context.selected_objects if o.type == 'MESH']

=== SECTION 3: BPY MESH VALIDATION ===

Getting mesh data from object:
  obj = bpy.context.selected_objects[0]   # first selected after import
  me = obj.data                            # bpy.types.Mesh

Vertex, polygon, triangle counts (bpy.types.Mesh, no bmesh needed):
  vert_count  = len(me.vertices)
  poly_count  = len(me.polygons)
  tri_count   = len(me.loop_triangles)    # call me.calc_loop_triangles() first if 0
  edge_count  = len(me.edges)
  # me.calc_loop_triangles() modifies me in-place, caches result in me.loop_triangles

UV map presence:
  has_uv = len(me.uv_layers) > 0
  uv_count = len(me.uv_layers)
  active_uv = me.uv_layers.active   # None if no UV layers

Manifold check via bmesh (preferred for headless — no operator context needed):
  import bmesh
  bm = bmesh.new()
  bm.from_mesh(me)
  bm.verts.ensure_lookup_table()
  bm.edges.ensure_lookup_table()
  non_manifold_edges = [e for e in bm.edges if not e.is_manifold]
  non_manifold_verts = [v for v in bm.verts if not v.is_manifold]
  is_manifold = len(non_manifold_edges) == 0 and len(non_manifold_verts) == 0
  bm.free()  # MANDATORY: free bmesh or memory leak

BMesh properties for manifold:
  edge.is_manifold   -> True if edge shared by exactly 2 faces
  edge.is_boundary   -> True if edge has exactly 1 face (open boundary)
  edge.is_wire       -> True if edge has 0 faces
  vert.is_manifold   -> True if vertex is fully surrounded by faces in a manifold configuration
  face.is_valid      -> True if face data is internally consistent

UV presence via bmesh:
  bm.loops.layers.uv.verify()     # returns active uv layer or creates one
  uv_layer = bm.loops.layers.uv.active  # None if no UV layers
  has_uv = uv_layer is not None

Face normals / orientation check:
  bm.faces.ensure_lookup_table()
  face_normals = [f.normal.copy() for f in bm.faces]
  # face.normal is a mathutils.Vector, auto-calculated from winding order
  # To check for flipped faces: compare face.normal with direction to camera/world center
  # No built-in "valid normal" bool — check normal magnitude ~= 1.0
  for f in bm.faces:
      assert abs(f.normal.length - 1.0) < 1e-4, f"degenerate face {f.index}"

NOTE: bpy.ops.mesh.select_non_manifold() requires EDIT mode and active context.
Do NOT use in headless background scripts. Use bmesh edge.is_manifold instead.

Bounding box (object space, no bmesh needed):
  # obj.bound_box is list of 8 Vector-like tuples in LOCAL space
  bb = obj.bound_box  # list of 8 [x, y, z] lists
  from mathutils import Vector
  world_bb = [obj.matrix_world @ Vector(v) for v in bb]
  xs = [v.x for v in world_bb]
  ys = [v.y for v in world_bb]
  zs = [v.z for v in world_bb]
  bbox_min = (min(xs), min(ys), min(zs))
  bbox_max = (max(xs), max(ys), max(zs))
  dimensions = obj.dimensions   # bpy.types.Object.dimensions: world-space bounding box size Vector

NOTE: obj.bound_box is in local object space. Always apply matrix_world to get world coordinates.
NOTE: obj.dimensions is world-space absolute size (accounts for scale), read-only in most contexts.

=== SECTION 4: BPY RENDERING ===

Camera setup:
  scene = bpy.context.scene
  cam_data = bpy.data.cameras.new("Camera")
  cam_obj = bpy.data.objects.new("Camera", cam_data)
  scene.collection.objects.link(cam_obj)
  scene.camera = cam_obj

  # Set position:
  cam_obj.location = (x, y, z)          # mathutils.Vector or 3-tuple

  # Set rotation (Euler XYZ in radians):
  import math
  cam_obj.rotation_euler = (math.radians(rx), math.radians(ry), math.radians(rz))
  cam_obj.rotation_mode = 'XYZ'         # default is 'XYZ'

Look-at via Track-To constraint (cleanest headless approach):
  import bpy
  from mathutils import Vector
  # Point camera at target location:
  direction = target_location - cam_obj.location
  rot_quat = direction.to_track_quat('-Z', 'Y')   # camera looks down -Z
  cam_obj.rotation_euler = rot_quat.to_euler()

Camera data settings:
  cam_data.lens = 50.0                  # focal length mm, default 50
  cam_data.type = 'PERSP'              # 'PERSP' | 'ORTHO' | 'PANO'
  cam_data.clip_start = 0.1
  cam_data.clip_end = 1000.0

Render settings:
  scene.render.engine = 'CYCLES'        # 'CYCLES' | 'BLENDER_EEVEE' | 'BLENDER_WORKBENCH'
  scene.render.resolution_x = 512       # int, pixels
  scene.render.resolution_y = 512       # int, pixels
  scene.render.resolution_percentage = 100  # int, 1-100
  scene.render.filepath = "/tmp/render_output"   # str, no extension (added by Blender)
  scene.render.image_settings.file_format = 'PNG'  # 'PNG'|'JPEG'|'OPEN_EXR'|'TIFF'
  scene.render.image_settings.color_mode = 'RGBA'  # 'BW'|'RGB'|'RGBA'
  scene.render.image_settings.color_depth = '8'    # '8'|'16' for PNG

Trigger render and write file:
  bpy.ops.render.render(write_still=True)
  # Writes to scene.render.filepath + extension based on file_format
  # PNG -> filepath + ".png"
  # With frame numbering: filepath = "/tmp/frame_####" -> /tmp/frame_0001.png

NOTE: EEVEE requires a display/GPU context. In true headless (no display), use CYCLES or WORKBENCH.
NOTE: CYCLES GPU in headless requires explicit GPU setup:
  scene.cycles.device = 'GPU'
  prefs = bpy.context.preferences.addons['cycles'].preferences
  prefs.compute_device_type = 'CUDA'   # or 'OPTIX', 'HIP', 'METAL'
  prefs.get_devices()

Headless safety: for validators (not rendering), use WORKBENCH — no GPU required, fastest.
For multi-angle renders: loop camera positions, update cam_obj.location/rotation_euler, call bpy.ops.render.render(write_still=True) each iteration.

=== SECTION 5: SCENE CLEANUP ===

Remove objects and meshes (batch processing — prevent memory accumulation):
  # Remove specific object:
  bpy.data.objects.remove(obj, do_unlink=True)
  # Remove mesh data (also removes referencing objects):
  bpy.data.meshes.remove(me, do_unlink=True)

  # Full scene clear:
  bpy.ops.wm.read_factory_settings(use_empty=True)  # resets to blank scene
  # OR selective clear:
  for o in list(bpy.data.objects):
      bpy.data.objects.remove(o, do_unlink=True)
  for m in list(bpy.data.meshes):
      bpy.data.meshes.remove(m)

Image cleanup (textures):
  for img in list(bpy.data.images):
      img.user_clear()
      bpy.data.images.remove(img)

bmesh cleanup (MANDATORY after every bmesh.new()):
  bm.free()

GOTCHA: bpy.data.objects.remove() on an object that still has users may not actually free memory. Use do_unlink=True.
GOTCHA: Objects deleted from scene stay in memory until file is closed if data-block has other users. Check .users property.
GOTCHA: Calling bpy.ops.wm.read_factory_settings() clears ALL data including scripts themselves — call at start of new iteration, not end of current.

=== SECTION 6: BLENDER FAILURE MODES ===

Malformed OBJ:
  - Blender 4.0+ C++ importer (wm.obj_import) is more tolerant than old Python importer.
  - Returns {'CANCELLED'} on complete parse failure, but may return {'FINISHED'} with partial geometry for lightly malformed files.
  - Always check len(bpy.context.selected_objects) > 0 after import.
  - Degenerate geometry (zero-area faces, duplicate verts) does not error — results in broken bmesh.

Malformed glTF/GLB:
  - import_scene.gltf returns {'CANCELLED'} on schema error.
  - Binary GLB with corrupt chunk headers causes Python exception (caught by --python-exit-code).
  - JSON-only .gltf with missing external bin files raises exception.

FBX corruption:
  - Old Python importer may raise exception on corrupt FBX (caught).
  - New C++ importer (wm.fbx_import) may segfault on corrupt binary FBX — exit code 139.

OOM behavior:
  - Blender does not handle OOM gracefully. OS OOM killer sends SIGKILL -> exit code 137.
  - Large meshes (>10M verts) or multiple 4K textures can trigger OOM.
  - No Python exception raised for OOM — subprocess just disappears.

Hung script:
  - Blender has no built-in script timeout. External process timeout (SIGKILL) is only recourse.
  - Infinite loops in bpy.ops calls possible if context is wrong.

Display requirement for EEVEE:
  - EEVEE requires GPU + display context. In headless without Xvfb, EEVEE raises:
    "Fatal Python error: Segmentation fault" or "Unable to open a display"
  - Use CYCLES --background or WORKBENCH for true headless.

=== SECTION 7: FFPROBE JSON OUTPUT ===

Command (audio stream metadata):
  ffprobe -v quiet -print_format json -show_streams -show_format -select_streams a:0 input.wav

JSON output structure (audio stream):
  {
    "streams": [{
      "codec_type": "audio",
      "codec_name": "pcm_s16le",       // string: codec identifier
      "sample_rate": "44100",           // string (NOT int): audio sample rate in Hz
      "channels": 2,                    // int: channel count
      "channel_layout": "stereo",       // string: layout name
      "bits_per_sample": 16,            // int: bit depth (reliable for WAV, FLAC)
      "bits_per_raw_sample": "16",      // string: raw bit depth — may be "N/A" or absent for lossy codecs
      "duration": "3.456000",           // string: duration in seconds (float)
      "bit_rate": "1411200",            // string: bits/sec — ABSENT for lossless FLAC
      "nb_frames": "152352"             // string: frame count (may be absent)
    }],
    "format": {
      "filename": "input.wav",
      "duration": "3.456000",           // string: container-level duration
      "size": "303158",                 // string: file size bytes
      "bit_rate": "1413696"             // string: container bitrate
    }
  }

CRITICAL: sample_rate is a STRING in JSON output, not an integer. Parse with int().
CRITICAL: bits_per_sample is the reliable field. bits_per_raw_sample may be absent or "N/A".
CRITICAL: bit_rate is ABSENT for lossless formats (FLAC). Check key existence before use.
CRITICAL: duration appears in both "streams" and "format". Stream-level is more precise.

Rust subprocess pattern:
  let output = Command::new("ffprobe")
    .args(["-v", "quiet", "-print_format", "json", "-show_streams", "-show_format",
           "-select_streams", "a:0", input_path])
    .output()?;
  // parse output.stdout as JSON

=== SECTION 8: FFPROBE SILENCE AND CLIPPING DETECTION ===

Silence detection command (writes to stderr):
  ffmpeg -i input.wav -af "silencedetect=n=-60dB:d=2" -f null -
  # n = noise threshold (default -60dB or amplitude 0.001)
  # d = minimum silence duration in seconds (default 2.0)
  # Output printed to stderr, NOT stdout

Silence detection stderr output format:
  [silencedetect @ 0x...] silence_start: 1.5
  [silencedetect @ 0x...] silence_end: 4.2 | silence_duration: 2.7

Parse pattern (Rust):
  Lines containing "silence_start:" -> silence onset timestamp (seconds)
  Lines containing "silence_end:" -> silence end + duration

Clipping detection command (astats filter):
  ffmpeg -i input.wav -af "astats=metadata=1:reset=1" -f null - 2>&1 | grep -E "clipping|Peak"
  # OR get machine-readable metadata:
  ffmpeg -i input.wav -af "astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.Number_of_clipping_samples" -f null -

astats metadata key format:
  lavfi.astats.<X>.<key>
  where X = channel number (1-based) or "Overall"
  Keys relevant to clipping:
    Number_of_clipping_samples   -> int count of samples that exceeded valid range
    Peak_level                   -> peak level in dBFS
    Crest_factor                 -> ratio of peak to RMS (not in dB)
    RMS_level                    -> RMS level in dBFS

Combined audio analysis command (single pass):
  ffmpeg -i input.wav \
    -af "silencedetect=n=-60dB:d=0.5,astats=metadata=1:reset=0,ametadata=print:key=lavfi.astats.Overall.Number_of_clipping_samples:key=lavfi.astats.Overall.Peak_level" \
    -f null - 2>&1

NOTE: astats outputs to stderr. Use 2>&1 redirect in shell, or capture stderr in Rust subprocess.
NOTE: silence and astats filters can be chained with comma in -af.

=== SECTION 9: FFMPEG KEYFRAME / FRAME EXTRACTION ===

Extract every Nth frame as PNG:
  ffmpeg -i input.mp4 -vf "select=not(mod(n\,N))" -vsync vfr output_%03d.png
  # N = frame interval (e.g., 30 = every 30th frame)
  # -vsync vfr: MANDATORY — prevents frame duplication

Extract actual I-frames (keyframes) only:
  ffmpeg -i input.mp4 -vf "select=eq(pict_type\,I)" -vsync vfr output_%03d.png

Extract frames at fixed time intervals (every T seconds):
  ffmpeg -i input.mp4 -vf "fps=1/T" output_%03d.png
  # e.g., fps=1/10 -> one frame every 10 seconds

Output naming:
  %03d -> 3-digit zero-padded (001, 002, ..., 999)
  %04d -> 4-digit zero-padded

Rust subprocess pattern:
  Command::new("ffmpeg")
    .args(["-i", input_path,
           "-vf", &format!("select=not(mod(n\\,{}))", frame_interval),
           "-vsync", "vfr",
           output_pattern])  // e.g. "/tmp/frames/frame_%03d.png"

NOTE: Backslash escaping of comma inside shell string: `select=not(mod(n\,N))`. In Rust args array, no shell escaping needed: `"select=not(mod(n,N))"` is passed directly to ffmpeg.

=== SECTION 10: BEHAVIORAL CONTRACTS ===

bpy.ops return values:
  All bpy.ops return a set: {'FINISHED'} = success, {'CANCELLED'} = aborted, {'RUNNING_MODAL'} = async.
  Check: result = bpy.ops.wm.obj_import(filepath=p); assert 'FINISHED' in result

Context requirement for operators:
  bpy.ops operators require a valid bpy.context. In headless --background mode, context IS available but limited.
  bpy.ops.mesh.* operators require EDIT mode. Use bmesh API instead in background scripts.
  bpy.ops.render.render() works in background mode with CYCLES and WORKBENCH engines.

bmesh thread safety:
  bmesh objects are NOT thread-safe. Create one bmesh per thread. Free after use.
  bm.from_mesh() copies mesh data. Changes to bmesh do NOT affect original mesh unless bm.to_mesh(me) called.

bpy.data side effects:
  Importing a file adds objects to bpy.data.objects AND bpy.context.scene.collection.
  bpy.context.selected_objects reflects the last import selection.
  After import, active object: bpy.context.view_layer.objects.active (may be None in headless).

=== SECTION 11: PLATFORM CONSTRAINTS ===

Windows:
  Blender executable: typically C:\Program Files\Blender Foundation\Blender X.X\blender.exe
  Path separator in Python scripts within Blender: use forward slashes or raw strings. Blender's bpy accepts both.
  subprocess on Windows: use full blender.exe path. Shell=False preferred to avoid quoting issues.

Linux/headless server:
  EEVEE requires Xvfb or real display. Set DISPLAY=:0 or use CYCLES/WORKBENCH.
  CYCLES CPU mode works with no display. GPU requires driver.

Blender version detection in script:
  bpy.app.version        # (major, minor, patch) tuple of ints
  bpy.app.version_string # "4.2.1" string

ffprobe/ffmpeg availability:
  Not bundled with Blender. Must be in PATH or provide absolute path.
  Version check: ffprobe -version (prints to stdout, exit 0)

=== SECTION 12: CRITICAL ASSERTIONS ===

assert 'FINISHED' in bpy.ops.wm.obj_import(filepath=path), "OBJ import failed"
assert len(bpy.context.selected_objects) > 0, "No objects imported"
assert all(o.type == 'MESH' for o in imported_objects), "Non-mesh objects imported"
assert len(bm.edges) > 0, "bmesh has no edges — geometry is degenerate"
assert all(abs(f.normal.length - 1.0) < 1e-3 for f in bm.faces if f.is_valid), "Degenerate face normals"
assert os.path.exists(render_output_path), "render write_still=True did not produce file"
assert isinstance(ffprobe_json["streams"][0]["sample_rate"], str), "sample_rate is string, must parse as int"
assert int(ffprobe_json["streams"][0]["sample_rate"]) > 0, "sample_rate must be positive"
assert ffprobe_json["streams"][0].get("bits_per_sample", 0) in (8, 16, 24, 32), "unexpected bit depth"

=== SECTION 13: IMPLEMENTATION PITFALLS ===

Do NOT use bpy.ops.import_scene.obj in Blender 4.0+. It does not exist. Use bpy.ops.wm.obj_import.
Do NOT use bpy.ops.mesh.select_non_manifold in --background headless scripts. It requires EDIT mode context. Use bmesh edge.is_manifold.
Do NOT call bm.free() before reading bm data. Free only after all reads are complete.
Do NOT use EEVEE engine in headless mode without display. It will segfault or error.
Do NOT rely on bpy.context.view_layer.objects.active after import in headless — it may be None. Use bpy.context.selected_objects instead.
Do NOT parse ffprobe sample_rate as int directly from JSON. The field is a JSON string "44100", not number.
Do NOT assume bits_per_raw_sample is present in ffprobe JSON. Use bits_per_sample for WAV/FLAC. Check key existence.
Do NOT use -vsync 0 with select filter in ffmpeg. Use -vsync vfr to prevent frame duplication.
Do NOT omit bm.free() after bmesh.new(). Memory leak accumulates per-call in batch processing.
Do NOT call bpy.ops.wm.read_factory_settings() mid-script if you need current scene data — it destroys everything.
Do NOT apply matrix_world transform after computing world_bb from raw bound_box — bound_box is already in local space, matrix_world IS the transform to world space. Apply once, not twice.
Do NOT use obj.dimensions for per-axis bounding box min/max. It gives size only, not min/max coords. Use matrix_world @ Vector(bb_point).
Do NOT set --python-exit-code 0 to "disable" error exit and then check exit codes. 0 disables the feature. Use a non-zero value like 1.
