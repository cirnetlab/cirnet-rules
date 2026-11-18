import unittest

from retry_count import retry_count


class RetryCountContract(unittest.TestCase):
    def test_decimal_values(self):
        for raw, expected in [("0", 0), (" 003 ", 3), ("+3", 3), ("0012", 12)]:
            with self.subTest(raw=raw):
                self.assertEqual(retry_count(raw), expected)

    def test_invalid_decimal_chains_the_parse_error(self):
        for raw in ("", "retry", "1.5"):
            with self.subTest(raw=raw):
                with self.assertRaises(ValueError) as result:
                    retry_count(raw)
                self.assertEqual(str(result.exception), "retry count must be a decimal integer")
                self.assertIsInstance(result.exception.__cause__, ValueError)

    def test_negative_value_has_its_own_error(self):
        with self.assertRaises(ValueError) as result:
            retry_count("-1")
        self.assertEqual(str(result.exception), "retry count cannot be negative")
        self.assertIsNone(result.exception.__cause__)
