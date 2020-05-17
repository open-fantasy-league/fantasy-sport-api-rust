FROM timescale/timescaledb:latest-pg12

# Unneeded. leafving in for future work/ref
ENV POSTGRES_DB leaderboard
ENV POSTGRES_USER fantasy
ENV POSTGRES_PASSWORD fantasy

COPY scripts/extensions.sh /docker-entrypoint-initdb.d/