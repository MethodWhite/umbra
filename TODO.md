# UMBRA — Estado Final

## Build
- Desktop: `cargo build --release --bin umbra-gui` ✅ 0 errors, 0 warnings
- Server: `cargo build --release --bin umbra --features server` ✅
- Synapsis: `cargo build --release --bin synapsis-mcp` ✅

## Tests: **107 total**
- umbra: 47 tests
- synapsis: 34 tests
- synapsis-core: 26 tests (22 unit + 4 benchmarks)

## Benchmarks (synapsis-core)
- Token efficiency: summary is **31%** of full content
- Search speed: **754µs** avg per query
- Token budget: **85% reduction** with tight budget
- Semantic search: related results rank **10x higher**

## CI/CD
- GitHub Actions: check + test + server + deny ✅
- Dependabot: weekly cargo, monthly actions ✅
- License: Umbra MIT, Synapsis BUSL-1.1 ✅
- Templates: CONTRIBUTING, CODE_OF_CONDUCT, ISSUE/PR templates ✅

## Assets
- Logo SVG (Umbra + Synapsis) ✅
- Architecture diagram SVG ✅
- Badges in README ✅
