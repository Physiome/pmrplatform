#!/bin/sh

set -e

# This is here to workaround bugs that make sqlx-cli annoying/impossible
# to use within a workspace using the simple commands - this script will
# prepare a sqlite database file with the crate root, specifying that as
# the source using absolute path to ensure the correct path (as for some
# reason it looks at the workspace root?) and remove the db, all this to
# ensure the offline build actually works.
#
# Run this script every time query! macros have their queries changed.
#
# For reference, these are (some of) the bugs
#
# - https://github.com/launchbadge/sqlx/issues/1223
# - https://github.com/launchbadge/sqlx/issues/1399
# - https://github.com/launchbadge/sqlx/issues/2441

cd "$(dirname "$0")"
DB_FILE=pmrmodel.db
echo -ne > ${DB_FILE} && cat migrations/pmr*/*sql | sqlite3 ${DB_FILE}
cargo sqlx prepare --database-url sqlite:$(pwd)/${DB_FILE} -- --tests
rm ${DB_FILE}
