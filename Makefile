BUILDDIR=$(PWD)/build
CRATE_DIR=$(PWD)
TARGET_DIR=$(CRATE_DIR)/target
CRATE_NAME=boson
DESTDIR=$(HOME)/.steam/root/compatibilitytools.d/$(CRATE_NAME)

.PHONY: build build-dev

all: pack-release
dev: pack-dev

build:
	cargo build --release

build-dev:
	cargo build

pack-release: prep build
	cp $(TARGET_DIR)/release/$(CRATE_NAME) $(BUILDDIR)/$(CRATE_NAME)
	@echo "Copying assets"
	cp -av $(CRATE_DIR)/assets/. $(BUILDDIR)

pack-dev: prep build-dev
	ln -fv $(TARGET_DIR)/debug/$(CRATE_NAME) $(BUILDDIR)/$(CRATE_NAME)
	@echo "Copying assets"
	cp -av $(CRATE_DIR)/assets/. $(BUILDDIR)


install: pack-release
	@echo "Installing to $(DESTDIR)"
	mkdir -p $(DESTDIR)
	cp -av $(BUILDDIR)/* $(DESTDIR)

prep:
	mkdir -p $(BUILDDIR)

clean:
	rm -rf $(BUILDDIR)