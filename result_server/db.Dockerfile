FROM postgres:latest
COPY scripts/extensions.sql /docker-entrypoint-initdb.d/