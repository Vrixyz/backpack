#!/usr/bin/env bash

set -v
set -eo pipefail

test_databases_file=./tmp_removed_test_dbs.txt

# suppressing `source .env` as it contains secrets!
set +v
source .env
set -v

touch $test_databases_file

PGPASSWORD=$PGPASSWORD PGUSER=$PGUSER psql -d postgres -c "\copy (SELECT datname FROM pg_database WHERE datname LIKE 'test_%' AND datistemplate=false) TO '$test_databases_file'" 

while read dbname
do
  echo "dropping DB $dbname..."
  PGPASSWORD=$PGPASSWORD PGUSER=$PGUSER dropdb -e "$dbname" 
done < $test_databases_file

echo "removing $test_databases_file file"
rm -r $test_databases_file