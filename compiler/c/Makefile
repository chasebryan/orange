# Standalone C frontend. Output stays under out/, which the repository ignore
# rules already exclude.
CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -Wpedantic -Werror -O2
CPPFLAGS ?= -Iinclude
LDFLAGS ?=

SRCS = src/main.c src/compile.c src/bigint.c src/typeparams.c src/pack.c src/tests.c src/amounts.c
OUT = out/orangec

.PHONY: all test clean

all: $(OUT)

$(OUT): $(SRCS) include/compile.h include/bigint.h include/orange.h include/typeparams.h include/pack.h include/tests.h include/amounts.h
	mkdir -p out
	$(CC) $(CPPFLAGS) $(CFLAGS) -o $(OUT) $(SRCS) $(LDFLAGS)

test: $(OUT)
	$(CC) $(CPPFLAGS) -DORANGEC_TEST -std=c11 -Wall -Wextra -Wpedantic -Werror -g -fsanitize=address,undefined,shift -fno-sanitize-recover=shift -fno-omit-frame-pointer -o out/orangec-asan $(SRCS) $(LDFLAGS)
	out/orangec --self-test
	out/orangec-asan --self-test
	python3 tests/differential.py out/orangec
	python3 tests/differential.py out/orangec-asan
	python3 tests/step_budget.py out/orangec
	python3 tests/step_budget.py out/orangec-asan

clean:
	rm -rf out
