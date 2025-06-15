#!/usr/bin/env python3

import sys
import os

pwd = os.getcwd()


def normalize(path):
    with open(path, 'r', encoding='utf-8') as f:
        return [line.replace("<DJWAVFIXER_PWD_PLACEHOLDER>", pwd).rstrip() for line in f]


found = normalize(sys.argv[1])
expected = normalize(f"./resources/cli_results/{sys.argv[1]}")

if found != expected:
    print("Files differ:")
    for line_number, (found_line, expected_line) in enumerate(zip(found, expected), 1):
        if found_line != expected_line:
            print(f"Line {line_number}:\n  expected: {expected_line}\n  got     : {found_line}")
    sys.exit(1)
else:
    print("Files match")
