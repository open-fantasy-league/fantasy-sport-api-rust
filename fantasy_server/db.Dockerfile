FROM postgres:latest
COPY scripts/extensions.sh /docker-entrypoint-initdb.d/