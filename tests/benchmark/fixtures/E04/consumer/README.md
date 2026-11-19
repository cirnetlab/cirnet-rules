# Card consumer

`contracts.lock.json` identifies the API artifact used by the consumer.
`generated/card.mjs` is the strict decoder emitted for that artifact. Do not
edit generated files. The independent fixture at `../producer` is the contract
owner and contains the release record.

Run the consumer tests with `node --test`. The sample wire response is recorded
in the released contract.
