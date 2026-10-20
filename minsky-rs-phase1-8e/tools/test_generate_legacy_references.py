import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("generator", Path(__file__).with_name("generate_legacy_references.py"))
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)

class GeneratorTests(unittest.TestCase):
    def test_detects_schema_versions(self):
        fixtures = Path(__file__).resolve().parents[1] / "tests/fixtures/legacy"
        self.assertEqual(generator.schema_version(fixtures / "schema1/exponentialGrowth.mky"), "1")
        self.assertEqual(generator.schema_version(fixtures / "schema2/exponentialGrowth.mky"), "2")

    def test_hash_matches_known_data(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "file"
            p.write_bytes(b"abc")
            self.assertEqual(generator.sha256(p), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")

if __name__ == "__main__":
    unittest.main()
