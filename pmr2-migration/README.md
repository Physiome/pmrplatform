# PMR2 migration

This contains a plan and collection of scripts/programs to help with migration
from PMR2.

## Scripts

- `pdbg_workspace_export.py`

  This script should be executed under the Zope/Plone debugger that provides
  the workspace data to export.

  The underlying workspace repo should be available.

- `src/bin/workspace.rs`

  This import program uses the pmrplatform's API to create the database entries
  and either copy/move/symlink the git repository to make available for use by
  the pmrplatform programs.
