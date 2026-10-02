# Native render-region ownership and measured style sharing

Status: ready-for-agent
Blocked by: none

Implement spec outcome 6. Trace current root construction and rejected cached-pane experiment. Design stable native region ownership that can actually reduce unrelated primitive construction without stale content, dropped input/native animations/async state/inherited style/layout or callback metadata. Favor intrinsic correctness over arbitrary public cache flags; definite-sized cache requirements must be explicit. Measure local Style/StoredNode/journal cost and immutable interning/reclamation using real workloads; retain supported optimization and eliminate failed experiments from production. Add machine-independent key work/correctness tests; retain serial bounded native performance baseline/candidate evidence under this ticket. Shared Cargo compilation may overlap other agents; defer reliable timing until the integrator can serialize native measurements. Do not declare no implementation solely because design is complex; implement a coherent useful region seam and prove it. Coordinate root/input/tree edits during merge, not by compromising invariants.
