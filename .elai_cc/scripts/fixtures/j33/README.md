# J-33 fixtures: MinerU doc-parser

`formula_stub.pdf` -- a minimal valid PDF whose content stream draws the
literal ASCII `E=mc^2` via a `BT /F1 12 Tf 10 60 Td (E=mc^2) Tj ET` text
object. Used as a read_to_string baseline for the harness-vs-HTML delta
and the indexer-parse delta.

The live TeX-from-MinerU delta is `pending_mineru_serve = true` in
`metrics/j33_mineru_fixture_eval.toml` until a real MinerU pipeline is
reachable. The fixture + the assertion shape ship today; the measured
numbers land when the service boots.

No external PDFs are copied here; the stub is self-contained and tiny
(~400 bytes) so it commits cleanly and exercises the harness without a
binary asset fetch.
