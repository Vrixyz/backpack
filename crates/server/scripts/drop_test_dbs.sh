#!/bin/bash

test_databases_file=$PWD/removed_test_dbs.txt
touch $test_databases_file
psql -d postgres -c "COPY (SELECT datname FROM pg_database WHERE datname LIKE 'test_%' AND datistemplate=false) TO '$test_databases_file'"

while read dbname
do
  echo "dropping DB $dbname..."
  dropdb "$dbname"
done < $test_databases_file

echo "removing $test_databases_file file"
rm $test_databases_file