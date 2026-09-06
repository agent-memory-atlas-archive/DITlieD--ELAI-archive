Dashboard and Frontend Application Rules

Loaded conditionally when the task involves dashboard development, data visualization, frontend UI components, or real-time state-driven interfaces. Every rule here is mandatory during plan creation and code execution.

When the surface's look/UX is not yet decided (no mock, brief, or reference), the component/state doctrine here has no design to apply yet: cook (.claude/skills/cook/SKILL.md) is the discovery front-end that pins the design read first. Offer it (cook is user-invoked only, never auto-fired) before building, then assemble from existing components per the rules below.


Component Architecture

1. NEVER concentrate API calls, state management, and rendering logic into a single parent component. This is the "God Component" anti-pattern. Distribute logic across: custom hooks for data fetching, context providers for shared state, and stateless components for rendering. If a single component file exceeds 150 lines, it must be decomposed.

2. NEVER pass methods from a bloated parent as individual props to children that already have access to the appropriate hooks or context. This creates chaotic coupling and "use case scattering" where modifying one behavior requires tracing through the parent and every child that received the prop.

3. Every stateful component, spawned element, or dynamically created UI artifact must have an explicitly defined cleanup path. Treat no UI element as permanent. If a component subscribes to a data stream, event listener, or interval, the corresponding unsubscribe/clearInterval/removeEventListener must exist in the cleanup return of the effect or in a destruction handler.


State Management

4. NEVER use useState for complex, interdependent state that updates rapidly (streaming data, real-time feeds, concurrent user interactions). useState introduces stale closures: rapidly executing functions capture outdated state references from previous render cycles, causing data overwrites and dropped updates. Use useReducer for any state where transitions depend on the current state value. The reducer function always receives current state from React's internal fiber tree, neutralizing the closure race condition.

5. NEVER write data fetching logic inside useEffect without implementing AbortController cleanup. When a user triggers rapid sequential requests (filter changes, pagination clicks), older slower requests may resolve after newer ones, overwriting the UI with stale data. Every fetch inside useEffect must: create an AbortController, pass its signal to the request, and abort in the cleanup return. Additionally, implement a loading state lock that prevents save/mutate operations until retrieval completes.

6. NEVER chain isolated useEffect hooks that react sequentially to isolated state changes (State A updates triggers Effect updating State B triggers Effect updating State C). This cascading synchronous render cycle freezes the browser main thread. Consolidate dependent state transitions into a single reducer or a single effect that batches updates.


Data Visualization

7. Select visualization libraries based on actual scale and interaction requirements:
   - D3.js: custom, research-heavy visuals requiring pixel-level SVG control. NEVER use D3 inside React without useRef isolation. D3 and React both demand DOM control; without explicit ref boundaries, memory leaks and rendering conflicts are guaranteed.
   - Recharts: declarative, component-based charts for standard React dashboards. Do not force complex interactive relationships onto its rigid composition model.
   - ECharts: massive datasets requiring zoom/pan/filter without lag. Use its built-in interaction system; do not write redundant custom event listeners over the canvas.
   - Chart.js: canvas-based, quick standard admin dashboards. Do not mix Canvas and SVG rendering methods.

8. NEVER render all data points simultaneously for large datasets (thousands of nodes). Implement: list virtualization (render only visible viewport), data decimation (downsample for overview, full resolution on zoom), and canvas rendering over SVG for datasets exceeding 1000 nodes. Naive rendering freezes the browser tab.

9. When implementing multi-chart synchronization (brushing, zooming on a primary chart updates a secondary chart), preserve data contract integrity through the formatting pipeline. Verify that the JSON data payload is not silently mutating: no swapped key/value pairs, no incorrect floating-point rounding, no lost analytical fields. Add assertions on the data shape before and after transformation.


Security

10. NEVER render user-uploaded or externally fetched SVG content without full sanitization. Validating that XML is parseable and the root node is <svg> is insufficient. You MUST strip: all event handler attributes (onload, onerror, onmouseover, etc.), all <script> elements, all javascript: URLs including variants with embedded whitespace or control characters that browsers auto-strip during URL parsing, and all <foreignObject> elements that could embed arbitrary HTML. Use a dedicated SVG sanitizer library, not string prefix checks.

11. NEVER trust that the frontend framework handles serialization/deserialization security automatically. When handling server-side data serialization (e.g., Server Components, server actions), implement rigorous type checking on all deserialized payloads. Unchecked deserialization is a direct path to remote code execution.

12. When generating plugin integrations, API connectors, or tool interfaces for dashboard applications, implement strict authorization boundary checks, type validation, and runtime payload inspection at every connection point. Do not assume the framework provides these by default.


Framework Awareness

13. Before generating code that uses any frontend framework (React, Next.js, Vue, Svelte) or CSS framework (Tailwind, etc.), verify the project's installed version and read the local documentation bundled in node_modules if available. Do not rely on training data for framework APIs. Deprecated features, renamed utilities, and changed configuration patterns cause immediate build failures. When in doubt, check package.json and run the framework's version command.

14. Do not generate dynamic, programmatically constructed CSS class names (e.g., 'bg-gradient-to-' + direction) when using utility-first CSS frameworks. Automated upgrade tools cannot detect or modify dynamic strings. Use complete, static class names that the framework's compiler can analyze.


Anti-Hallucination

15. NEVER reference component properties, API endpoints, or library methods without verifying they exist in the installed version. Fabricated props and endpoints do not crash immediately; they silently produce undefined behavior, broken layouts, or security holes that surface only in production.

16. When tasked with debugging a dashboard component, do not generate fixes that introduce new issues. Verify that every variable rename cascades to all references, every import resolves to an installed package, and every function call targets an existing method. "Ghost imports" for uninstalled libraries are a build-breaking defect.


Audit Checklist (for Adversarial Auditor)

When auditing dashboard or frontend code, the auditor MUST check for:
- God Components concentrating fetch, state, and render logic in one file
- useState used for rapidly updating concurrent state instead of useReducer
- Missing AbortController cleanup in async data fetching effects
- Cascading useEffect chains creating synchronous render waterfalls
- D3 direct DOM manipulation inside React without useRef isolation
- Large dataset rendering without virtualization or decimation
- Unsanitized SVG content (missing strip of event handlers, script elements, javascript: URLs)
- Unchecked deserialization of server-side payloads
- Dynamic CSS class name construction that breaks framework compilation
- Fabricated component props, API endpoints, or library methods
- Missing cleanup handlers for subscriptions, intervals, or event listeners
- Multi-chart synchronization that silently mutates data shape
