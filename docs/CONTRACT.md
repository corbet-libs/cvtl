# Implemented Volatile contract

`Volatile<C, N = RedisNetwork>` selects Memory or Valkey at service startup.
Constructors route Scope, Limits, Clock and the supplied network to the leaf.
The facade owns no record, index, clock floor, retry policy or member lifecycle.

API: the shared cmmr::Store trait (`get`, `compare_exchange`, `delete_generation`,
`page`, `maintain`, `close`, `state`). Types and errors are re-exported unchanged.
States Closed / Ready / Unavailable are read directly from the chosen child.
Arc-backed clones share both data and readiness. There is no fallback or SQL.
Warnings delegate to cvlk; reopen/reconciliation is performed on its child.

Domain owners supply deadlines, CAS revisions, generation names and indexes.
Each handle is scoped to a trusted community and service incarnation. Facade
selection does not authenticate a caller, encrypt data, admit a member or decide
query policy. Memory is process-local; Valkey shares state between service
processes on the supplied primary connection. cthl's existing throttle API and
cvlk's adapter are exposed for composition, without duplicating a quota ledger.

One unchanged cmmr conformance suite is run through both variants. Tests prove
expiry, isolation, atomic CAS and index replacement, generation fencing,
capacity, cursor binding and lifetime, cancellation-before-poll, regression and
close propagation. cvlk adds real network fault, native expiry, throttle and
load coverage; cmmr executes identical native/wasm vectors. This facade is native
server code. Future Attend/Lookup end-to-end acceptance awaits those domain
libraries; storage conformance does not claim forum admission is implemented.
