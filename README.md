# Linux client

A simple client that:

1. implements communication over WebSocket using [`generic-client`](https://github.com/mpclipboard/generic-client)
2. integrates with Wayland clipboard to read/write clipboard text
3. shows a tray icon with 5 last clips (implements `org.kde.StatusNotifierItem` spec)

### Building

```
meson setup builddir --buildtype=release --prefix=/usr
meson compile -C builddir
meson install -C builddir --destdir=$PWD/test-install
tree test-install

test-install/
└── usr
    ├── bin
    │   └── mpclipboard-client
    └── lib
        └── systemd
            └── user
                └── mpclipboard-client.service
```
