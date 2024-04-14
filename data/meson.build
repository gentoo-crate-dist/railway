subdir('icons')

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('app-id', application_id)
desktop_conf.set('bindir', bindir)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  type: 'desktop',
  po_dir: podir,
  install: true,
  install_dir: join_paths(datadir, 'applications')
)

if desktop_file_validate.found()
  test('Validate desktop file', desktop_file_validate,
    args: [desktop_file]
  )
endif

# Metainfo
appdata_conf = configuration_data()
appdata_conf.set('base-id', base_id)
appdata_conf.set('app-id', application_id)
appdata_conf.set('gettext-package', gettext_package)
appdata = configure_file(
  input: '@0@.metainfo.xml.in'.format(base_id),
  output: '@0@.metainfo.xml'.format(application_id),
  configuration: appdata_conf,
  install: true,
  install_dir: datadir / 'metainfo'
)

if appstream_cli.found()
  test('Validate metainfo file', appstream_cli,
    args: ['validate', '--no-net', '--explain', appdata]
  )
endif

# Schemas
install_data(
  '@0@.gschema.xml'.format(base_id),
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

# Compile schemas
settings_schemas = [ '@0@.gschema.xml'.format(base_id) ]
gnome.compile_schemas(depend_files: files(settings_schemas))

subdir('resources')