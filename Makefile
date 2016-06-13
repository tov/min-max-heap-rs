default: build
hard: test

CRATE = min_max_heap
REPO  = min-max-heap-rs

build:
	clear
	cargo build
	make doc

clippy:
	rustup run nightly cargo build --features=clippy

doc:
	cargo doc --no-deps
	echo "<meta http-equiv='refresh' content='0;url=$(CRATE)/'>" > target/doc/index.html

test:
	clear
	cargo test

upload-doc:
	make doc
	ghp-import -n target/doc
	git push -f https://github.com/tov/$(REPO).git gh-pages

release:
	make upload-doc
	cargo publish

clean:
	cargo clean
