#!/usr/bin/env bash
set -euo pipefail
umask 077
cd /root/Aether
exec 9>/run/lock/aether-deploy.lock
flock -n 9 || { echo 'Another deployment is running'; exit 1; }
command="${SSH_ORIGINAL_COMMAND:-${1:-}}"
if [[ "${command}" == check ]]; then
  uname -m
  docker compose -f compose.json ps
  exit 0
fi
[[ ! -e MIGRATION_IN_PROGRESS ]] || { echo 'Migration has not finished'; exit 1; }
[[ "${command}" =~ ^deploy\ ([a-f0-9]{40})$ ]] || { echo 'Expected deploy followed by a full commit SHA'; exit 1; }
revision="${BASH_REMATCH[1]}"
image="ghcr.io/zouni6666/aether:sha-${revision}"
docker pull "${image}"
[[ "$(docker image inspect --format '{{.Architecture}}' "${image}")" == arm64 ]]
[[ "$(docker image inspect --format '{{index .Config.Labels "org.opencontainers.image.revision"}}' "${image}")" == "${revision}" ]]
backup="/var/backups/aether-deploy/$(date -u +%Y%m%dT%H%M%SZ)-${revision:0:12}"
install -d -m 700 "${backup}"
cp .env app.env compose.json "${backup}/"
previous="$(docker inspect --format '{{.Image}}' aether-app)"
printf '%s\n' "${previous}" > "${backup}/previous-image"
docker exec aether-postgres sh -lc 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc -Z 3' > "${backup}/database.dump"
docker exec aether-redis redis-cli --rdb /data/deployment-backup.rdb >/dev/null
docker cp aether-redis:/data/deployment-backup.rdb "${backup}/redis.rdb"
if ! APP_IMAGE="${image}" docker compose -f compose.json up -d --no-deps --wait --wait-timeout 300 app; then
  echo "Health check failed; restoring previous application image. Database backup: ${backup}"
  APP_IMAGE="${previous}" docker compose -f compose.json up -d --no-deps --wait --wait-timeout 300 app
  exit 1
fi
printf 'APP_IMAGE=%s\n' "${image}" > .env.next
mv .env.next .env
printf '%s\n' "${revision}" > deployed-revision
docker compose -f compose.json ps app
echo "Deployed ${revision}; backup: ${backup}"
