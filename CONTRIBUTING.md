# Contributing

Keep geometric rules in the library and argument/file handling in the binary.
Use `cargo fmt`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`.

Packing changes must preserve every item, permitted orientations, margins, and
minimum spacing. Include a small CSV that demonstrates the change. Do not claim
optimal packing unless it is established; report heuristic improvements with
their limitations. Avoid adding dependencies without a clear application benefit.

Use focused commits with imperative subjects. Pull requests should explain the
problem, resulting behavior, relevant issue, and checks performed. Include only
original or properly licensed sample inventories and artwork. Never commit secrets
or private customer information. State whether physical printing was actually tested.
