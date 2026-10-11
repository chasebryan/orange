# Standalone C frontend. Output stays under out/, which the repository ignore
# rules already exclude.
#
# `make test` builds with address and undefined sanitizers, runs the
# exact-integer self-test, and compares check, eval, and lex with the Rust
# orangec through tests/differential.py.
CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -Wpedantic -Werror -O2
CPPFLAGS ?= -Iinclude
LDFLAGS ?=

SRCS = src/main.c src/compile.c src/bigint.c
OUT = out/orangec

.PHONY: all test clean

all: $(OUT)

$(OUT): $(SRCS) include/compile.h include/bigint.h
	mkdir -p out
	$(CC) $(CPPFLAGS) $(CFLAGS) -o $(OUT) $(SRCS) $(LDFLAGS)

test: $(OUT)
	$(CC) $(CPPFLAGS) -std=c11 -Wall -Wextra -Wpedantic -Werror -g -fsanitize=address,undefined -fno-omit-frame-pointer -o out/orangec-asan $(SRCS) $(LDFLAGS)
	out/orangec --self-test
	out/orangec-asan --self-test
	python3 tests/differential.py

clean:
	rm -rf out
