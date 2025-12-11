# PMR2 migration

This contains a plan and collection of scripts/programs to help with migration
from PMR2.

## Scripts

- `pdbg_workspace_export.py`

  This script should be executed under the Zope/Plone debugger that provides
  the workspace data to export.

  The underlying workspace repo should be available.

- `pdbg_exposure_export.py`

  This script should be executed under the Zope/Plone debugger that provides
  the exposure data to export.

- `src/bin/workspace.rs`

  This import program uses the pmrplatform's API to create the database entries
  and either copy/move/symlink the git repository to make available for use by
  the pmrplatform programs.

- `src/bin/exposure.rs`

  This import program uses the pmrplatform's API to create the database entries
  and either copy/move/symlink the git repository to make available for use by
  the pmrplatform programs.


## Steps

- Run the above Python scripts on the PMR2 instance to extract the relevant
  entries from the underlying ZODB.
- Initialize the default access control policiesby running `./pmrac.sh`.
- Initialize the default profiles running `./profiles/import.sh`; ensure the
  setup is done correctly by reading `./profiles/README.md`.
- Then run the import programs (workspace, then exposure) to create the
  database entries for `pmrplatform.`
