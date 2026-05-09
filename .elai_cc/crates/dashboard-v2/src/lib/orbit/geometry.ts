// orbit/geometry.ts
// Polar/cartesian helpers, curved Bezier path builder, deterministic
// angle distributors, and viewBox math for zoom-to-node transitions.

export interface Point {
  x: number;
  y: number;
}

export interface NodeBounds {
  x: number;
  y: number;
  r: number;
}

export interface ViewportRect {
  x: number;   // top-left x in svg coords
  y: number;   // top-left y in svg coords
  w: number;
  h: number;
}

// Polar -> cartesian. theta in radians, r in svg units.
export function polar(theta: number, r: number, cx = 0, cy = 0): Point {
  return { x: cx + r * Math.cos(theta), y: cy + r * Math.sin(theta) };
}

// Quadratic Bezier path from (x1,y1) to (x2,y2) bowed by `bowFactor`
// fraction of chord length. Returns SVG path string starting with "M".
export function curve(x1: number, y1: number, x2: number, y2: number, bowFactor = 0.18): string {
  const mx = (x1 + x2) / 2;
  const my = (y1 + y2) / 2;
  const dx = x2 - x1;
  const dy = y2 - y1;
  const dist = Math.sqrt(dx * dx + dy * dy);
  const perpX = -dy / (dist || 1);
  const perpY = dx / (dist || 1);
  const bow = dist * bowFactor;
  const cx = mx + perpX * bow;
  const cy = my + perpY * bow;
  return `M ${x1} ${y1} Q ${cx} ${cy} ${x2} ${y2}`;
}

// Cubic Bezier path with two control points at perpendicular offsets.
// Used for smoother edges in research-tree mode.
export function curveCubic(x1: number, y1: number, x2: number, y2: number, bowFactor = 0.22): string {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const dist = Math.sqrt(dx * dx + dy * dy);
  const perpX = -dy / (dist || 1);
  const perpY = dx / (dist || 1);
  const bow = dist * bowFactor;
  const c1x = x1 + dx * 0.33 + perpX * bow;
  const c1y = y1 + dy * 0.33 + perpY * bow;
  const c2x = x1 + dx * 0.66 + perpX * bow;
  const c2y = y1 + dy * 0.66 + perpY * bow;
  return `M ${x1} ${y1} C ${c1x} ${c1y} ${c2x} ${c2y} ${x2} ${y2}`;
}

// Distribute n angles symmetrically around a base theta within an arc.
// If arcSpan is full circle (2*pi), returns evenly-spaced ring angles.
// If arcSpan is partial (e.g. pi/3), returns angles around `baseTheta`.
export function distributeAngles(n: number, baseTheta: number, arcSpan: number): number[] {
  if (n <= 0) return [];
  if (n === 1) return [baseTheta];
  if (arcSpan >= 2 * Math.PI - 1e-6) {
    // full circle: evenly distributed starting at baseTheta
    const out: number[] = [];
    for (let i = 0; i < n; i++) out.push(baseTheta + (2 * Math.PI * i) / n);
    return out;
  }
  // partial arc: centered on baseTheta
  const step = arcSpan / (n - 1);
  const start = baseTheta - arcSpan / 2;
  const out: number[] = [];
  for (let i = 0; i < n; i++) out.push(start + step * i);
  return out;
}

// Golden-angle distributor — used for constellations (multi-concept).
// Phyllotaxis: angles follow the golden ratio so neighbours minimize overlap.
export function goldenAngles(n: number, baseTheta = -Math.PI / 2): number[] {
  const PHI = Math.PI * (3 - Math.sqrt(5)); // ~2.39996
  const out: number[] = [];
  for (let i = 0; i < n; i++) out.push(baseTheta + PHI * i);
  return out;
}

// Compute target viewBox for zoom-to-node. `padding` is fraction of node
// diameter to add as breathing room. Returns top-left + width/height.
export function nodeBoundsToViewport(node: NodeBounds, padding = 0.12): ViewportRect {
  const pad = node.r * 2 * padding;
  const side = node.r * 2 + pad * 2;
  return { x: node.x - side / 2, y: node.y - side / 2, w: side, h: side };
}

// Linear interpolate two viewports (for smooth zoom).
export function lerpViewport(a: ViewportRect, b: ViewportRect, t: number): ViewportRect {
  return {
    x: a.x + (b.x - a.x) * t,
    y: a.y + (b.y - a.y) * t,
    w: a.w + (b.w - a.w) * t,
    h: a.h + (b.h - a.h) * t,
  };
}

// Cubic ease-out — premium feel, matches tokens.css --ease-out cubic-bezier(0.16,1,0.3,1).
export function easeOutCubic(t: number): number {
  const c = Math.max(0, Math.min(1, t));
  return 1 - Math.pow(1 - c, 3);
}

// Format a viewBox as the string SVG expects.
export function viewportToString(v: ViewportRect): string {
  return `${v.x} ${v.y} ${v.w} ${v.h}`;
}

// -----------------------------------------------------------------
// Constellation anchor — where to place concept #i in a multi-concept
// task. For 1 concept: centre. For 2: left/right at +/- 220 px on x.
// For 3+: golden-angle distribution around the centre at radius 280.
// Anchors are deterministic given `n`.
// -----------------------------------------------------------------
export function constellationAnchor(i: number, n: number, cx = 320, cy = 280): Point {
  if (n <= 1) return { x: cx, y: cy };
  if (n === 2) {
    const dx = 240;
    return { x: cx + (i === 0 ? -dx : dx), y: cy };
  }
  // 3+: golden-angle around the centre at a fixed radius
  const r = 300;
  const PHI = Math.PI * (3 - Math.sqrt(5));
  const theta = -Math.PI / 2 + PHI * i;
  return { x: cx + Math.cos(theta) * r, y: cy + Math.sin(theta) * r };
}
