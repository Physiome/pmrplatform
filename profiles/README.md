# Exposure File View Profiles

The following instructions is assumed to be executed in the project root directory.

- Copy `profile/env.example` to `profile/env`
- Edit `profile/env` to point to the actual locations of:

  `PMR2_BUILDOUT_DIR`
      Point to the root of the `git clone` of `https://github.com/PMR2/pmr2.buildout`.

  `CMLIBS_VENV_DIR`
      This is the virtualenv with `cmlibs`, `cmlibs.argon` and friends installed.

  `PMRPLATFORM_DIR`
      This is the root of the `git clone` of `https://github.com/Physiome/pmrplatform/`.

- The placeholder values may be safely used but no exposure file views may be created for the
  associated profiles.
- Run `profile/import.sh` to import all profiles to `pmrplatform`.
