FROM timescale/timescaledb:latest-pg12
COPY scripts/extensions.sql /docker-entrypoint-initdb.d/