project('mpclipboard-client')

subdir('systemd')

find_program('cargo')
cargo_args = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
if get_option('buildtype') == 'release'
    cargo_args += ['--release']
endif

custom_target(
    'mpclipboard-client',
    command: [
        'sh',
        '-c', 'cargo build @0@ && cp @1@ @OUTPUT@'.format(
            ' '.join(cargo_args),
            meson.project_source_root() / 'target' / get_option('buildtype') / 'mpclipboard-client',
        ),
    ],
    output: 'mpclipboard-client',
    build_always_stale: true,
    build_by_default: true,
    install: true,
    install_dir: join_paths(
        get_option('prefix'),
        get_option('bindir'),
    ),
    console: true,
)
