#! /bin/sh
rainfrog \
  --username=airflow_user \
  --password=password \
  --host=localhost \
  --port=5432 \
  --database=airflow_db \
  --driver=postgres
