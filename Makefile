.DEFAULT_GOAL := check

.PHONY: check check-policy test-policy

check: test-policy check-policy

check-policy:
	python3 tools/validate_foundation.py

test-policy:
	python3 -m unittest discover -s tools/tests -p 'test_*.py'
