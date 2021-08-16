#!/bin/bash

RUSTFLAGS="-Z instrument-coverage" LLVM_PROFILE_FILE="stroka-%m.profraw" \
cargo test --tests \
  && cargo profdata -- merge -sparse stroka-*.profraw -o stroka.profdata \
  && cargo cov -- report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile=stroka.profdata \
  --summary-only \
  && cargo cov -- show \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --use-color \
  --ignore-filename-regex='/.cargo/registry' \
  --instr-profile=stroka.profdata \
  --show-instantiations \
  --show-line-counts-or-regions \
  --Xdemangler=rustfilt \
  --format=html > stroka-coverage.html
