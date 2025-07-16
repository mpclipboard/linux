project('mpclipboard-client')

subdir('systemd')

custom_target(
    'mpclipboard-client',
    command: [
        files('meson-build-rust.sh'),
        get_option('buildtype'),
        meson.project_source_root(),
        '@OUTPUT@',
    ],
    output: 'mpclipboard-client',
    build_always_stale: true,
    build_by_default: true,
    install: true,
    install_dir: join_paths(
        get_option('prefix'),
        get_option('bindir'),
    )
)
