blueprints = custom_target('blueprints',
  input: files(
    'ui/journey_store_item.blp',
    'ui/search_store_item.blp',
    'ui/place_list_item.blp',
    'ui/frequency_label.blp',
    'ui/alt_label.blp',
    'ui/indicator_icons.blp',
    'ui/journeys_page.blp',
    'ui/provider_list_item.blp',
    'ui/refresh_button.blp',
    'ui/remark_item.blp',
    'ui/search_options_button.blp',
    'ui/search_page.blp',
    'ui/time_divider.blp',
    'ui/transition.blp',
    'ui/journey_list_item.blp',
    'ui/shortcuts.blp',
    'ui/preferences_dialog.blp',
    'ui/provider_popover.blp',
    'ui/window.blp',
    'ui/journey_detail_page.blp',
    'ui/intermediate_location_item.blp',
    'ui/leg_item.blp',
    'ui/search_options_dialog.blp',
    'ui/station_entry.blp',
    'ui/date_time_picker.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

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
  dependencies: [ appdata, blueprints ],
)
