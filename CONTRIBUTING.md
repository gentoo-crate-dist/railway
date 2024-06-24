# Contributing

Contributions to Railway are always welcome. There are many different ways you can contribute, this document will show you most of these ways and what you should keep in mind.

We are interested in getting to know our contributors and are often around to discuss the state and the future of Railway, be it something minor, or larger visions. Feel free to join our [Matrix channel](https://matrix.to/#/#railwayapp:matrix.org) and talk with us! This is also a good place to ask some questions or report smaller issues.

Note that the [GNOME Code of Conduct](https://wiki.gnome.org/Foundation/CodeOfConduct) applies to this project, therefore, be nice to each other.

## Translation

This is probably the easiest way to contribute to Railway. Just head over to [Weblate](https://hosted.weblate.org/projects/schmiddi-on-mobile/railway/) and start translating. If you are unfamiliar with Weblate, make sure to read their [documentation](https://docs.weblate.org/en/latest/user/translating.html) on how to translate a project.

## Open Issues

Issues are a good way to tell me problems you are having with the applications or things that you feel are missing or might be improved. There are a few things to keep in mind with issues.

- If you think it could in any way be relevant, add your current version, and how you installed it.
- Add the logs if you think they are relevant, this is mostly usefull for errors that occurred.
- Check for duplicated issues: Try to use the search-feature if you can find similar issues like you are having. If there is already such an issue, consider giving it a thumbs-up or commenting more details on that issue, but don't create a new issue.
- Know how to write a good issue: Read e.g. <https://wiredcraft.com/blog/how-we-write-our-github-issues/> (also applies pretty much got GitLab)

## Write Code

If you feel comfortable enough writing code, you can also submit your changes directly via a merge request. Here are a few pointers that might help write the code:

### Compilation

#### GNOME Builder

You should be able to directly clone, compile and run Railway using [GNOME Builder](https://apps.gnome.org/Builder/). Note that if you installed GNOME Builder as a Flatpak, it cannot automatically install the dependencies Railway requires; you need to manually install them in that case using:

```bash
flatpak remote-add --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo  # Add the GNOME nightly repository, see <https://wiki.gnome.org/Apps/Nightly>
flatpak install org.gnome.Sdk//master
flatpak install org.freedesktop.Sdk.Extension.rust-stable//23.08
```

#### Linux

Compile Railway using the following commands:

```bash
meson build -Dprofile=development   # Run once before all changes you make. Substitute "development" for "default" for compiling for release.
meson compile -C build   # Run every time you want to test your changes
GSETTINGS_SCHEMA_DIR=./build/data/ RUST_LOG=diebahn=trace ./build/target/debug/diebahn  # Run your locally compiled application with some logging
```

This also requires installation of the dependencies from your package manager. Meson will inform you about any missing packages. Note that Railway tries to keep up-to-date with its dependencies, this might mean that the dependencies provided by fixed-release distros may be outdated. In that case, use one of the other ways of compilation mentioned below.

#### Flatpak via fenv

As an alternative, [fenv](https://gitlab.gnome.org/ZanderBrown/fenv) allows you to set up a flatpak
environment from the command line and execute commands in that environment.

First, install fenv:

```sh
# Clone the project somewhere on your system
git clone https://gitlab.gnome.org/ZanderBrown/fenv.git

# Move into the folder
cd fenv

# Install fenv with Cargo
cargo install --path .
```

You can now discard the `fenv` directory if you want.

After that, move into the directory where you cloned Railway and set up the project:

```sh
# Setup the flatpak environment
fenv gen build-aux/de.schmidhuberj.DieBahn.Devel.json

# Launch a shell inside the build environment
fenv shell
```

You can now follow the compilation phase for GNU/Linux

### Some useful documentation

The following documentation might help:

- [GTK Book](https://gtk-rs.org/gtk4-rs/stable/latest/book/): General GTK-development.
- [railway-backend](https://gitlab.com/schmiddi-on-mobile/railway-backend): The backend library for getting information about the journeys. Specifically, take a look at ["Writing a Provider"](https://gitlab.com/schmiddi-on-mobile/railway-backend/-/blob/main/docs/writing-a-provider.md?ref_type=heads) if you plan to work on a provider that is not yet implemented.

### Things to keep in mind

- Make sure the code passes some basic checks (`cargo check`).
- Make sure the code is properly formatted (`cargo fmt`).
- Reach out to me for bigger changes, either in an issue or via Matrix.
- Optional but encouraged: Check for new warnings in `cargo clippy`.
- Always feel free to ask questions or request some help.
