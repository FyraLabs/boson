DESTDIR=$(PWD)/build
CRATE_DIR=$(PWD)
TARGET_DIR=$(CRATE_DIR)/target
CRATE_NAME=boson

.PHONY: build build-dev

all: pack-release
dev: pack-dev

build:
	cargo build --release

build-dev:
	cargo build

pack-release: prep build
	cp $(TARGET_DIR)/release/$(CRATE_NAME) $(DESTDIR)/$(CRATE_NAME)
	@echo "Copying assets"
	cp -av $(CRATE_DIR)/assets/. $(DESTDIR)

pack-dev: prep build-dev
	ln -fv $(TARGET_DIR)/debug/$(CRATE_NAME) $(DESTDIR)/$(CRATE_NAME)
	@echo "Copying assets"
	cp -av $(CRATE_DIR)/assets/. $(DESTDIR)


prep:
	mkdir -p $(DESTDIR)

clean:
	rm -rf $(DESTDIR)