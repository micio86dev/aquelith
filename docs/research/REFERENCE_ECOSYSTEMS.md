# AQELITH Reference Ecosystems: Concept Extraction

Access date for all URLs: 2026-11-05. Status tags: **[V]** verified against the fetched primary source; **[A]** assumption or inference (mine, not stated by the source); **[C]** contradicted or qualified by evidence; **[G]** gap (not covered by what I could read; needs follow-up).
These are references, not requirements. No syntax, branding, or benchmark numbers are copied. Page-size caveat: several specs (LLVM LangRef, DOM, Fetch, HTML workers, CORE-AAM) were only partially read (first ~100k chars); claims are limited to what was read.

**Version pinning (limitation).** Most sources here are living documents (`main` branches, living standards, versionless docs sites). No commit hash, release number or snapshot id was recorded for them, so every claim holds only as of the access date above and must be re-checked against the then-current text before it is quoted in a normative AQELITH specification. Under `AGENTS.md` this document needs URL + access date, plus a document version where the site exposes one; where a source exposed none (or I did not record it), that is a known gap and no version has been invented.

---

## Rust

### Key ideas
- Ownership: one owner per value; value dropped at owner scope end; assignment of heap-owning values is a move that invalidates the source. [V] https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
- Compile-time-checked memory management with no runtime cost: "None of the features of ownership will slow down your program while it's running." [V] same URL
- Borrowing: either one `&mut` or any number of `&`, never both; references must not dangle; data races are rejected at compile time. Borrow lifetimes end at last use (non-lexical). [V] https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
- Traits: static dispatch via monomorphization (MIR is generic; monomorphization occurs at LLVM-IR lowering). [V] https://rustc-dev-guide.rust-lang.org/overview.html
- `unsafe` is a proof-obligation marker: unsafe items state conditions the compiler does not check; unsafe blocks/impls assert discharge. [V] https://doc.rust-lang.org/reference/unsafe-keyword.html
- Compiler architecture: AST -> HIR (type check, trait solving) -> THIR -> MIR (CFG; borrow check runs here) -> LLVM-IR. Query-based, on-disk cached incremental compilation; as of Nov 2022 only HIR->LLVM-IR were query-fied. [V] rustc-dev-guide overview. Pinned date means current state is [G].
- Borrow checking on a dedicated mid-level IR rather than on the AST/source is the reusable architectural idea. [V]/[A]

### Evidence / sources
- Book ch04-01, ch04-02; Reference "unsafe keyword"; Reference "behavior considered undefined" https://doc.rust-lang.org/reference/behavior-considered-undefined.html; rustc-dev-guide overview.
- UB list (data races, dangling/misaligned access, aliasing-rule breaks, invalid values, ABI violations, runtime-assumption violations) "is not exhaustive"; "There is no formal model of Rust's semantics for what is and is not allowed in unsafe code." [V] Reference UB page. Directly relevant: Rust's safety claim has no complete formal semantics for its unsafe boundary.

### Trade-offs and poor fits
- Learning cost and conservatism: the rules reject some valid programs (the Reference calls the type system "a conservative approximation of the dynamic safety requirements", used to explain why e.g. doubly-linked lists need unsafe). [V]
- Safety is conditional on unsafe code being sound; the unsafe boundary lacks a formal model. [V]
- Compile time: monomorphization plus LLVM; no benchmark claims made here. [G]
- Poor fit if AQELITH wants a low barrier for UI/application authors: pervasive lifetimes in application code (e.g. graph-shaped UI state, callbacks) are friction. [A]

### Implications for AQELITH
- Adopt the idea of a typed mid-level IR on which ownership/aliasing analysis runs; keep the checker separate from codegen so the same analysis serves AI tooling (diagnostics as structured data). [A]
- Make the unsafe/trusted boundary a first-class, enumerable, auditable construct with per-use obligation text; consider machine-readable obligations for AI agents. [A]
- Decide up front whether AQELITH ships a formal semantics for its safe subset; Rust's gap (above) is the cautionary example. [A]
- Consider a graduated model (default value semantics + opt-in unique ownership/borrows) rather than mandatory borrow checking for all code. [A]

---

## Swift

### Key ideas
- Value semantics by default for structs/enums, with copy-on-write for large value types; this is described as "generally a success" but costs refcounting, uniqueness checks, and forces heap allocation because copying can happen at any time. [V] https://github.com/swiftlang/swift/blob/main/docs/OwnershipManifesto.md
- Opt-in ownership: `~Copyable` types have unique ownership; `consume` invalidates a value; `deinit` runs at end of unique lifetime; `discard self` skips deinit. [V] https://github.com/swiftlang/swift-evolution/blob/main/proposals/0390-noncopyable-structs-and-enums.md
- Law of Exclusivity: overlapping accesses to a variable may not occur unless both are reads; enforced statically where storage allows (locals, inout, value-type properties), dynamically for class properties/globals (with a stated performance cost), undefined for unsafe pointers. [V] Ownership Manifesto
- Design stance: ownership features must not impose "substantial burdens on regular programs"; move-only types are for experts. [V]
- Native platform integration is via Objective-C/C interop and ABI stability. [A] (not fetched; Swift book page returned no content: [G])

### Evidence / sources
- Ownership Manifesto (URL above); SE-0390 (URL above). Swift book "Structures and Classes" fetch returned only a heading: [G].

### Trade-offs and poor fits
- As of SE-0390, noncopyable types could not be generic arguments, conform to most protocols, or be used in `Optional`/`Array`; adding or removing `Copyable` is ABI-breaking. [V] (state may have advanced since SE-0390; later proposals not read: [G])
- Hybrid static/dynamic exclusivity enforcement adds runtime cost where static proof is impossible. [V]
- "Shared" borrowed values are not first-class (cannot be stored/returned) in the manifesto design, deliberately limiting expressiveness. [V]
- Poor fit: retrofitting ownership onto a language with implicit copyability caused generics/ABI constraints. AQELITH has no legacy and should avoid the retrofit. [A]

### Implications for AQELITH
- Design the generics system and ABI with "copyable" as a trait/capability from day one, not a retrofit. [A]
- Value semantics as default with explicit, visible copy cost; avoid hidden heap allocation and hidden uniqueness checks on hot paths. [A]
- Choose static exclusivity where possible; if dynamic checks exist, make them opt-in/visible and removable under a profile (embedded/real-time). [A]
- Treat native platform bridging as a core design topic, not a library afterthought. [A]

---

## Kotlin Multiplatform

### Key ideas
- Shared common code plus platform-specific code via `expect`/`actual`; compiler enforces an actual for every expect on every platform, same package, no implementation in expect. [V] https://kotlinlang.org/docs/multiplatform/multiplatform-expect-actual.html
- Official guidance prefers ordinary interfaces over expect/actual: "rely on standard language constructs wherever possible"; expect/actual are mainly for DI configuration. [V] same URL
- Intermediate source sets (e.g. shared iOS code) allow hierarchical sharing. [V]
- Kotlin/Native C interop: header-driven binding generation (cinterop); pointers as `CPointer<T>?`, `memScoped` for scoped native memory, `nativeHeap` for manual; callbacks via `staticCFunction` must not capture; user data via manually disposed `StableRef`. [V] https://kotlinlang.org/docs/native-c-interop.html
- Swift/ObjC interop exists but details were not on the fetched page. [G]

### Evidence / sources
- The two URLs above.

### Trade-offs and poor fits
- Expected/actual classes were Beta and need a compiler flag; visibility mismatch rules; extra enum constants force `else` branches. [V] (as of the docs fetched)
- C-interop import is Beta; `reinterpret` casts are unsafe; pointers become invalid after `memScoped`; function-like macros need manual wrapping; no implicit integer casts. [V]
- Poor fit: binding-per-platform model produces compile-time conditional APIs; the community's own guidance steers away from it. A single unified platform abstraction needs a stronger capability model. [A]

### Implications for AQELITH
- Prefer capability/interface-based platform abstraction; if a compile-time platform switch exists, make completeness checking (every platform has an implementation) a compiler guarantee, as KMP does. [A]
- Generate FFI bindings from machine-readable interface descriptions, keep FFI memory-scope rules explicit and checkable (scope-bound pointers, non-capturing callbacks). [A]
- Treat the C/ObjC/Swift/JVM boundaries as explicit unsafe-ish edges with documented obligations. [A]

---

## Qwik

### Key ideas
- Resumability: server serializes listeners, component boundaries, subscriptions (store-to-component relationships), and application state into HTML; the client continues without hydration. [V] https://qwik.dev/docs/concepts/resumable/
- Handlers as URLs: a QRL is a URL of form `./chunk.js#SymbolName`; the optimizer makes each closure its own chunk/symbol; a single tiny global listener (Qwikloader) resolves the QRL and loads the chunk on interaction. [V] https://qwik.dev/docs/advanced/qrl/
- Closure captures: captured variables are encoded as an index array at the end of the QRL, pointing into the serialized object table (`q:obj` / `qwik/json`); `useLexicalScope()` restores them. [V] qrl page
- Serializable beyond JSON: circular references, DOM refs, Promises (via resources), Date, URL, Map, Set. [V] resumable page
- Not serializable: class instances (no prototype/`instanceof`), streams; such code must run client-only. Closures serialize only if wrapped in QRL. [V] resumable page
- Serialized state is a snapshot; attributes "may have stale values" while the app runs and are updated only when re-serialized. [V] qrl page
- Claims ~1KB JS to become interactive and "near zero initial JavaScript"; speed comes from skipping hydration work, not shipping less total code. [V] https://qwik.dev/docs/concepts/think-qwik/ (figures are vendor claims; no independent benchmark read: [G])
- Version handling: each container records its runtime version (`q:version`); containers may run different framework versions; components within a container must be compiled together and share the same bundle artifacts and version; containers can be deployed independently. [V] https://qwik.dev/docs/advanced/containers/

### Evidence / sources
- URLs above; deployments page https://qwik.dev/docs/deployments/ discusses long-term caching of content-hashed build files and warns that translated bundles can keep identical filenames across builds even when content changes. [V]

### Trade-offs and poor fits
- Deployment skew (old HTML/serialized state vs new bundles): the docs read only state version-per-container and content-hashed filenames; no stated mechanism for migrating serialized state across versions or for guaranteeing old HTML finds old chunks. [G] This is the key unknown to resolve before adopting the model. A reasonable inference: old HTML works only if old chunks stay deployed (immutable, retained). [A]
- Every closure boundary becomes a serialization boundary: developers must reason about what is capturable. Docs say it "requires a different way of thinking" and that existing frameworks cannot retrofit resumability without breaking compatibility. [V]
- Serialization gaps (class instances, streams) force a restricted data model. [V]
- Many tiny chunks imply many requests/prefetch heuristics; prefetching is not covered in the fetched page. [G]
- Poor fit if AQELITH targets non-browser deployments (native, embedded): the HTML-as-snapshot form is web-specific, but the principle is general. [A]

### Implications for AQELITH
- A language-level "serializable closure" capability (compiler-checked: captured values must be serializable types) would turn Qwik's runtime failures into compile errors. [A]
- Stable symbol identity for code (content-hash + name) enables lazy loading, resumption, and sandboxed handler dispatch. [A]
- Define the serialization format and versioning contract explicitly: include code version in snapshots, refuse or migrate on mismatch; never leave skew to caching luck. [A]
- Distinguish snapshot state from live state in the type system (docs show staleness as a footgun). [A]

---

## Svelte

### Key ideas
- Compiler-driven reactivity. Legacy (Svelte <=4) mode: `$:` statements are re-run on dependency change; dependencies are determined statically by what the statement reads, and the compiler "cannot 'see'" indirect dependencies through function calls. [V] https://svelte.dev/docs/svelte/legacy-reactive-assignments
- Svelte 5 runes (`$state`, etc.) are "symbols that you use ... to control the Svelte compiler"; they are language-level, need no import, and cannot be stored or passed around. [V] https://svelte.dev/docs/svelte/what-are-runes
- `$state` gives deep reactive proxies for arrays/plain objects; class instances are not proxied (use fields); destructured values evaluate once; arguments are passed by value so use getters/objects to carry liveness. [V] https://svelte.dev/docs/svelte/$state
- Cross-module limits: state exported from `.svelte.js/.ts` must not be reassigned; export an object or getters. [V]
- Svelte 5 moved from purely static compile-time dependency detection toward a runtime signal-like model: [A] (the fetched runes/state pages do not describe the runtime; [G] for the exact runtime design).

### Evidence / sources
- Three URLs above.

### Trade-offs and poor fits
- Reactivity that depends on syntactic visibility breaks across function/module boundaries (legacy), and proxies/special rules surface in Svelte 5 (destructuring, export). [V]
- Compiler magic couples semantics to the compiler: runes are not first-class values. [V]
- Poor fit: language-level reactivity bound to one UI domain; AQELITH is a general language. [A]

### Implications for AQELITH
- Dependency tracking can be a compiler analysis on a typed IR rather than runtime magic, but its limits (indirection) must be specified, not discovered. [A]
- Keep reactive primitives first-class and typed (so they cross module boundaries) rather than syntactic. [A]
- Beware reactive wrappers (proxies) that change identity/equality semantics; require explicit snapshots for serialization (cf. `$state.snapshot`). [V]/[A]

---

## Solid

### Key ideas
- Fine-grained reactivity: signals (getter/setter) and subscribers (effects/memos); a "current subscriber" variable registers dependencies on read; setters notify only when value changes. [V] https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity
- Components run once; only JSX/tracking scopes re-run. Reads outside a tracking scope do not subscribe. [V] https://docs.solidjs.com/concepts/intro-to-reactivity
- Updates target the specific attribute rather than re-running the component; contrasts explicitly with React re-execution. [V] first URL
- Memos cache and return signals; early returns stop tracking skipped reads. [V]
- Synchronous tracking: reads inside async callbacks (e.g. `setTimeout`) may not be tracked; use `on` or resources. [V]
- Updates synchronous by default; `batch` coalesces. [V]

### Evidence / sources
- The two URLs above. Ownership/disposal semantics of the reactive graph were not on the fetched pages: [G].

### Trade-offs and poor fits
- Tracking is dynamic and dependent on execution context; async boundaries lose it, which is a classic footgun. [V]
- Destructuring/escaping reads from tracking scopes loses reactivity (exact Solid doc on props destructuring not read: [G]).
- Poor fit for AQELITH core: dynamic dependency tracking is runtime-dependent and hard to prove deterministic or real-time bounded. [A]

### Implications for AQELITH
- Fine-grained update granularity is the useful concept: dependency graph at attribute/field level. [A]
- If AQELITH has reactive primitives, define async/tracking semantics formally (explicit dependency lists as a safe default, as `on` does). [A]
- For embedded/real-time profiles, reactive propagation needs bounded work per update; Solid's model does not guarantee this. [A]

---

## LLVM (and Cranelift)

### Key ideas
- Single SSA-based IR used across compilation phases, in three equivalent forms (in-memory, bitcode, text); typed; verifier checks well-formedness (e.g. definitions dominate uses). [V] https://llvm.org/docs/LangRef.html
- Poison/undef/UB model: violating attributes such as `nonnull`/`align` yields poison; ABI mismatches yield undefined behavior; `noalias` encodes aliasing promises. [V] (partial read; dedicated undef/UB sections not read: [G])
- Pass infrastructure: passes implement `run()` and return `PreservedAnalyses`; analysis manager caches results (e.g. dominator tree); required vs optional passes; `optnone` skips optional ones. [V] https://llvm.org/docs/WritingAnLLVMNewPMPass.html
- Frontends lower to LLVM IR; Rust does so after monomorphization. [V] rustc-dev-guide
- Cranelift: smaller scope, single IR covering several LLVM lowering levels, no assemblers/disassemblers, "biases ... towards fast compilation when mid-level optimization is not needed", currently no mid-level optimizations; integers limited to i8..i64, no aggregate types. [V] https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/compare-llvm.md (doc may be dated; current e-graph optimizer not covered: [G])

### Evidence / sources
- URLs above. No compile-time or code-quality benchmarks used.

### Trade-offs and poor fits
- LLVM IR's poison/UB semantics give optimizers freedom but make frontend correctness subtle: a frontend that emits attributes it cannot guarantee gets miscompiles. [A] grounded in [V] attribute-violation text.
- LLVM is large and slow-to-compile relative to Cranelift (qualitatively stated by Cranelift docs; no numbers). [V]
- Cranelift lacks the optimization depth and target breadth of LLVM; fits JIT/fast-debug, not peak-performance release builds. [V]/[A]
- Poor fit: depending solely on LLVM binds AQELITH to its release cadence, size, and IR semantics. [A]

### Implications for AQELITH
- Own a high-level typed IR (where ownership, effects, and capabilities are checked) and lower to a backend IR; keep the backend swappable (LLVM for release, a fast backend for dev/JIT/AI-iteration loops). [A]
- Model analysis-preservation and pass ordering explicitly if AQELITH exposes an optimization pipeline. [A]
- Define the language's own UB policy: either no UB in safe code (preferred) or enumerated and documented; emit backend attributes (`noalias`, `nonnull`) only when proven by AQELITH's checker. [A]

---

## WebAssembly and WASI

### Key ideas
- Sandboxing: modules run in an environment separated from the host and interact only via defined APIs; control-flow integrity (declared call targets, protected call stack, runtime signature check on indirect calls); linear memory is bounds-checked at region level; traps on abnormal behavior. [V] https://webassembly.org/docs/security/
- WASI is capability-based: "The Wasm language has no syscall instructions or built-in I/O facilities". Link-time capabilities (imports) and runtime capabilities (unforgeable first-class handles); virtualizable/attenuable by wrapping components. [V] https://github.com/WebAssembly/WASI/blob/main/docs/Capabilities.md
- Component model: WIT interface language, canonical ABI mapping abstract values to core Wasm values, shared-nothing (only `module` currently crosses), resource types with `own`/`borrow` handles and optional destructors, uniqueness/dropping enforced at runtime by canonical definitions. [V] https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md
- Component model status: features beyond the WASI 0.2 preview (async, stream/future, `map`, fixed-length lists) may be disabled by default or change; some combinations rejected (e.g. `borrow` in stream/future). [V] same URL
- WASI preview 2 docs page fetched returned 404: [G].

### What WASM does NOT guarantee about security (primary source)
- Memory bugs inside linear memory: "Data stored in linear memory can overwrite adjacent objects" (region-level bounds checks, not per-object). [V]
- Code-reuse attacks on indirect calls remain possible at function granularity. [V]
- Race conditions (TOCTOU) are possible; no scheduling guarantees beyond in-order execution and atomics. [V]
- Side channels (including timing) can occur. [V]
- Randomization and bounded-pointer mitigations are proposals, not provided. [V] (page is dated; current state of mitigations not checked: [G])

### WASM vs JavaScript performance: evidence
- Primary peer-reviewed evidence found: Jangda et al., "Not So Fast" (USENIX ATC 2019): WebAssembly vs native code on SPEC CPU, "slower by an average of 45% (Firefox) to 55% (Chrome)", peak slowdowns 2.08x and 2.5x. [V] https://www.usenix.org/conference/atc19/presentation/jangda
- That abstract makes no WASM-vs-JS claim. [C] Common claims that WASM is generally faster than JS are therefore not established by the source I read; I did not locate a primary, current WASM-vs-JS comparison. [G] No numbers are asserted here. Data is also 2019-era browsers; engines have changed: [G].

### Trade-offs and poor fits
- Strong isolation at the module boundary, weak protection inside the module (see above). [V]
- Shared-nothing components imply copying across boundaries (canonical ABI lift/lower). [V]/[A] cost not measured: [G].
- Poor fit as AQELITH's only target: no direct DOM/GC/threads parity assumed; WASI maturity and component-model features still in flux. [V]/[A]

### Implications for AQELITH
- Use WASM/WASI as a compile target and a sandbox for untrusted plugins/agents, but do not rely on it for in-module memory safety: AQELITH's own language-level safety must hold inside the module. [A]
- Adopt capability-based I/O (handles, attenuation by wrapping) as the language-level security model for both native and WASM targets. [A]
- Mirror the own/borrow handle notion for cross-boundary resources; plan an interface description language comparable to WIT. [A]
- Publish performance claims only from own, reproducible benchmarks. [A]

---

## Web standards

### Key ideas
- DOM: tree model with preorder depth-first tree order; shadow trees with slots; event dispatch phases (capture, at-target, bubble) with `stopPropagation`/`preventDefault`; closed shadow roots hide nodes from `composedPath()`. [V] https://dom.spec.whatwg.org/ (partial read; MutationObserver detail: [G])
- HTML custom elements: lifecycle callbacks, upgrade of pre-existing elements on definition, form-associated elements, `attachInternals()` for default ARIA roles/states/custom CSS states (`:state()`); shadow can be disabled. [V] https://html.spec.whatwg.org/multipage/custom-elements.html
- Web Workers: separate global scope and event loop, `postMessage` with structured clone, transferable `ArrayBuffer`, MessagePort transfer, SharedWorker per origin; "relatively heavy-weight". [V] https://html.spec.whatwg.org/multipage/workers.html (explicit no-DOM statement not in the portion read: [G])
- Fetch: unified request/response model; request modes (same-origin, cors, no-cors, navigate, websocket), credentials modes; filtered/opaque responses protect cross-origin data; atomic redirects; streams for bodies; forbidden headers and `Sec-`/`Proxy-` reserved. [V] https://fetch.spec.whatwg.org/ (partial read; CORS algorithm not read: [G])
- WebSocket: no built-in backpressure; only `bufferedAmount`; no ping/pong exposure; failures reported uniformly (close code 1006) so scripts cannot probe the local network. [V] https://websockets.spec.whatwg.org/
- CSS cascade: origin and importance, then layers (`@layer`), then specificity, then order; layers scoped to shadow contexts; important declarations invert order. [V] https://www.w3.org/TR/css-cascade-5/ (partial)
- Accessibility tree: browsers map semantics to platform accessibility APIs (MSAA/UIA/AT-SPI/AX/Android); ARIA overrides native semantics except where host language conflicts; accessible name/description, state-change events are part of the mapping. [V] https://www.w3.org/TR/core-aam-1.2/ (partial)

### Trade-offs and poor fits
- DOM/CSS carry a very large legacy surface (mutation, global cascade, specificity) that is hard to analyze statically. [A]
- WebSocket without backpressure pushes flow control onto apps. [V]
- Worker messaging via structured clone constrains transferable data to cloneable types (same constraint class as Qwik serialization). [V]/[A]
- Poor fit as direct language semantics: AQELITH should not bake DOM or CSS cascade semantics into its core. [A]

### Implications for AQELITH
- Treat the accessibility tree as a first-class output of UI declarations (semantics-first: role/name/state), not retrofitted from rendered pixels or markup. [A]
- Model isolation boundaries (workers, shadow trees, origins) as explicit communication-by-message with typed, serializable payloads. [A]
- Offer a web target that emits standards-compliant DOM/ARIA/custom elements; keep native targets mapped to platform accessibility APIs through the same semantic layer (Core-AAM shows the mapping pattern). [A]
- Network APIs should have backpressure and cancellation by design (Fetch controller/abort, streams) rather than WebSocket's model. [A]

---

## Embedded and real-time (Rust no_std as the reference)

### Key ideas
- `no_std` links `core` instead of `std`; excludes dynamic allocation (`Box`, `Vec`), file/network capabilities; `core` is platform-agnostic and keeps primitives, atomics, SIMD. [V] https://doc.rust-lang.org/reference/names/preludes.html and https://docs.rust-embedded.org/book/intro/no-std.html
- Runtime services of `std` (stack overflow protection, argument handling, spawning the main thread before `main`) are absent. [V]
- Heap is opt-in via the `alloc` crate plus a chosen global allocator; `HashMap/HashSet` unavailable without a secure RNG. [V]
- Layered library design (core / alloc / std) is the reusable architectural pattern. [V]/[A]
- RTOS and real-time ecosystems (RTIC, Embassy, Zephyr, FreeRTOS), panic strategy, and WCET/determinism guidance: not covered by the fetched pages. [G]

### Evidence / sources
- The two URLs above. The embedded "Start" page returned no relevant content for panic/RTOS: [G].

### Trade-offs and poor fits
- Losing `std` fragments the ecosystem (crates must support `no_std`). [A]
- Allocator choice affects determinism; real-time guarantees are outside what these sources state. [G]
- Poor fit: a single runtime profile cannot serve both web apps and microcontrollers. [A]

### Implications for AQELITH
- Define standard-library tiers from day one (no-allocation core, allocation-enabled, OS-hosted) with enforced dependency direction. [A]
- Make allocation, panics/aborts, and blocking visible in effect/capability types so real-time profiles can forbid them at compile time. [A]
- Reactive UI features must degrade to bounded-time update paths under real-time profiles. [A]

---

## Cross-cutting implications
1. Own a typed high-level IR where ownership, effects, capabilities, and serializability are checked; lower to swappable backends (Rust MIR, LLVM/Cranelift split). [A]
2. Make "copyable", "serializable", and "sendable across boundary" explicit traits/capabilities from day one (Swift retrofit cost, Qwik serialization gaps, Worker structured clone, WASM canonical ABI). [A]
3. Capability-based I/O and plugin isolation (WASI) plus language-level memory safety inside modules (WASM does not provide it). [A]
4. Auditable, enumerable trust boundaries (unsafe, FFI, host calls) with machine-readable obligations, good for AI agents. [A]
5. Standard-library tiers and effect visibility for embedded/real-time. [A]
6. Semantics-first UI with the accessibility tree as a first-class output. [A]
7. Explicit versioning for any persisted/serialized artifact or code identity (resumability, ABI, component interfaces). [A]
8. Specify rather than discover limits of any compile-time analysis (Svelte legacy deps, Rust UB gaps). [A]

## Open questions
1. How does Qwik (and similar resumable systems) handle deployment skew between cached HTML/serialized state and new bundles? Docs read do not say. Requires source or maintainer design docs.
2. Is there a current, primary, reproducible WASM-vs-JavaScript performance study? None found; only a 2019 native comparison.
3. What is the current status of Swift noncopyable generics and Rust's formal semantics (e.g. Stacked/Tree Borrows) beyond the fetched docs?
4. What runtime model does Svelte 5 use for dependency tracking, and how does it differ from Solid's?
5. Cranelift: current mid-end (e-graph) optimization status and measured trade-offs vs LLVM.
6. Real-time guarantees: what do RTIC, Embassy, Zephyr, and FreeRTOS actually specify (WCET, priority ceilings, allocation policy)?
7. WASI preview 2/3 and component-model async: stable surface and timeline for AQELITH target planning.
8. Primary DOM/Fetch/HTML sections not read (MutationObserver, CORS algorithm, worker DOM restriction) should be completed before AQELITH web-target design.
9. Which parts of AQELITH require a formal semantics, and who owns it?
