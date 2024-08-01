bootlib:
	$(MAKE) --directory=boot

userlib:
	$(MAKE) --directory=user

all:bootlib userlib	
	cd rust_main && cargo build

test:bootlib userlib
	cd rust_main && cargo build --features test_linkme	

.PHONY:clean
clean:
	$(MAKE) --directory=user clean
	$(MAKE) --directory=boot clean
	cd rust_main && cargo clean