#! /bin/sh
podman run \
  -dt \
  --rm \
  --name airflow_postgresql \
  --env POSTGRES_USER=airflow_user \
  --env POSTGRES_PASSWORD=password \
  --env POSTGRES_DB=airflow_db \
  --volume airflow_postgresql:/var/lib/postgresql/data \
  --publish 5432:5432 \
  docker.io/library/postgres:alpine
