FROM timescale/timescaledb:latest-pg12
COPY scripts/extensions.sh /docker-entrypoint-initdb.d/