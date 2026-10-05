"""Include the standalone Orange Book checks in normal unittest discovery."""
import importlib.util
from pathlib import Path
import unittest


def load_tests(loader: unittest.TestLoader, tests: unittest.TestSuite,
               pattern: str | None) -> unittest.TestSuite:
    path = Path(__file__).resolve().parents[1] / 'test_book_foundations.py'
    spec = importlib.util.spec_from_file_location('orange_book_checks', path)
    if spec is None or spec.loader is None:
        raise ImportError(f'cannot load book checks: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return loader.loadTestsFromModule(module)
