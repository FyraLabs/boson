DESTDIR=$(PWD)/build
CRATE_DIR=$(PWD)
TARGET_DIR=$(CRATE_DIR)/target
CRATE_NAME=boson
CARGO_ARGS=""
CARGO_TARGET="release"

.PHONY: build build-dev

all: pack-release
dev: pack-dev

build:
	cargo build --release

build-dev:
	cargo build

pack-release: prep build
	cp $(TARGET_DIR)/$(CARGO_TARGET)/$(CRATE_NAME) $(DESTDIR)/$(CRATE_NAME)
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

package: pack-release
	@# copy to tmp
	mkdir -p /tmp/$(CRATE_NAME)
	cp -av $(DESTDIR)/. /tmp/$(CRATE_NAME)/$(CRATE_NAME)
	tar -C /tmp/$(CRATE_NAME) -caf $(CRATE_NAME).tar.zst $(CRATE_NAME)
	rm -rf /tmp/$(CRATE_NAME)