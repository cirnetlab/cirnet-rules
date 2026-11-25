def _parse_count(text):
    return int(text, 10)


def retry_count(text):
    try:
        result = _parse_count(text)
    except ValueError as exc:
        raise ValueError("retry count must be a decimal integer") from exc
    if result < 0:
        raise ValueError("retry count cannot be negative")
    return result
