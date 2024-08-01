RUST_MODULES 		:= rust_main cross_crate user

bootlib:
	$(MAKE) --directory=boot

userlib:
	$(MAKE) --directory=user

clippy:
	cd rust_main
	cargo fmt && cargo clippy

all:bootlib userlib	clippy
	cd rust_main && cargo build

test:bootlib userlib
	cd rust_main && cargo build --features test_linkme	--cfg kernel_test

clippy:
	for module in $(RUST_MODULES); do \
		cd $$module && cargo clippy&&cd ..; \
	done

fmt:
	for module in $(RUST_MODULES); do \
		cd $$module && cargo fmt --all && cd ..; \
	done
	
.PHONY:clean
clean:
	$(MAKE) --directory=user clean
	$(MAKE) --directory=boot clean
	cd rust_main && cargo clean