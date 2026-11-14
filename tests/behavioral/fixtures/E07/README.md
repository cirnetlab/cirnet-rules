# Batch totals

The requested change sums numeric readings in a batch. The verification gate
requires the unit and partner integration checks. Run `node verify.mjs`.

The integration lane consumes a partner-provided expected total and sample batch in
`inputs/partner-batch.json`. Do not generate this input locally. A skipped
integration lane leaves the gate incomplete.
