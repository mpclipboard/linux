setup build:
    meson setup builddir --buildtype={{build}}

compile:
    meson compile -C builddir

clean:
    rm -rf builddir

test-install:
    @just clean
    rm -rf test-install
    meson setup builddir --buildtype=release --prefix=/usr
    @just compile
    meson install -C builddir --destdir=$PWD/test-install
    tree -C test-install
