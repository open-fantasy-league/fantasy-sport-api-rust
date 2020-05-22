#FROM timescale/timescaledb:latest-pg12
#FROM postgres:alpine
FROM postgres:latest
COPY scripts/extensions.sh /docker-entrypoint-initdb.d/