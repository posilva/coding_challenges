.PHONY: run setup clean_cover cover check

run:
	cargo run -- "$@"

setup:
	cargo install grcov

clean_cover:
	rm -rf *.profraw ./target/coverage

cover: clean_cover
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='ccwc-%p-%m.profraw' cargo test && \
					  grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html && \
					  open ./target/coverage/html/index.html

check:
	cargo clippy
