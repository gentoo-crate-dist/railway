# Resources
resources_conf = configuration_data()
resources_conf.set('app-id', application_id)
resources = gnome.compile_resources(
  'resources',
  configure_file(
    input: 'resources.gresource.xml.in',
    output: '@BASENAME@',
    configuration: resources_conf,
    install: true,
    install_dir: datadir / 'metainfo'
  ),
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
  dependencies: [ appdata ],
)
