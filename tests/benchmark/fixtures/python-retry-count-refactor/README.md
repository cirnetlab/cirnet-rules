# Retry count parser

`retry_count.py` parses a non-negative decimal retry count. Whitespace and a
leading plus sign are accepted. Invalid decimal input raises
`ValueError("retry count must be a decimal integer")` with the original parse
error as its cause. A negative count raises
`ValueError("retry count cannot be negative")` without a cause.

From this directory, verify with
`python -B -m unittest discover -s qa -p contract_*.py`. Use bounded local
inspection of the Python source and its tests. Keep local notes, drafts and
ignored files outside inspection output and unchanged.
