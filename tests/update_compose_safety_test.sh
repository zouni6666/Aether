#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf -- "${TEST_ROOT}"' EXIT

fail_test() {
    echo "FAIL: $*" >&2
    exit 1
}

make_fixture() {
    local fixture="$1"
    mkdir -p "${fixture}/bin" "${fixture}/compose"
    : >"${fixture}/compose/docker-compose.yml"
    : >"${fixture}/calls"
    cp "${REPO_ROOT}/update.sh" "${fixture}/compose/update.sh"
    chmod 0755 "${fixture}/compose/update.sh"
}

make_migration_docker_stub() {
    local fixture="$1"
    cat >"${fixture}/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${AETHER_TEST_CALLS}"
case "$*" in
    "compose version"|"info") exit 0 ;;
    *"config --services") printf 'app\n' ;;
    *"pull app") exit 0 ;;
    *"run --rm --no-deps -T app --migrate") exit "${AETHER_TEST_MIGRATION_STATUS:-0}" ;;
    *"up --help") printf '%s\n' '      --wait  Wait for services' ;;
    *"up -d --wait --wait-timeout 120 app"|*" ps") exit 0 ;;
    *) printf 'Unexpected docker arguments: %s\n' "$*" >&2; exit 99 ;;
esac
EOF
    chmod 0755 "${fixture}/bin/docker"
}

test_migration_succeeds_before_recreate() {
    local fixture="${TEST_ROOT}/migration-before-recreate"
    make_fixture "${fixture}"
    make_migration_docker_stub "${fixture}"

    PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" \
        >"${fixture}/stdout" 2>"${fixture}/stderr"

    local pull_line migration_line recreate_line
    pull_line="$(grep -nF 'pull app' "${fixture}/calls" | cut -d: -f1)"
    migration_line="$(grep -nF 'run --rm --no-deps -T app --migrate' "${fixture}/calls" | cut -d: -f1)"
    recreate_line="$(grep -nF 'up -d --wait --wait-timeout 120 app' "${fixture}/calls" | cut -d: -f1)"
    [[ "${pull_line}" -lt "${migration_line}" && "${migration_line}" -lt "${recreate_line}" ]] \
        || fail_test "update did not pull, migrate, then recreate in order"
    [[ "$(grep -Fc ' --migrate' "${fixture}/calls")" -eq 1 ]] \
        || fail_test "update did not run migrations exactly once"
    ! grep -Eq 'db prepare|apply-backfills|--service-ports' "${fixture}/calls" \
        || fail_test "migration preflight enabled backfills or published service ports"
}

test_migration_failure_preserves_running_app() {
    local fixture="${TEST_ROOT}/migration-failure"
    make_fixture "${fixture}"
    make_migration_docker_stub "${fixture}"

    if PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        AETHER_TEST_MIGRATION_STATUS=42 \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" \
        >"${fixture}/stdout" 2>"${fixture}/stderr"; then
        fail_test "update succeeded after schema migration failed"
    fi

    grep -Fq 'run --rm --no-deps -T app --migrate' "${fixture}/calls" \
        || fail_test "update did not attempt schema migration"
    ! grep -Eq ' (up|stop|down|restart|rm) ' "${fixture}/calls" \
        || fail_test "failed migration changed the running service"
    grep -Fq 'app was not recreated' "${fixture}/stderr" \
        || fail_test "migration failure did not explain that app was left running"
    ! grep -Fq '>>> Done.' "${fixture}/stdout" \
        || fail_test "failed migration was reported as successful"
}

test_prepare_only_pulls_image() {
    local fixture="${TEST_ROOT}/prepare-only"
    make_fixture "${fixture}"
    make_migration_docker_stub "${fixture}"

    PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" --prepare \
        >"${fixture}/stdout" 2>"${fixture}/stderr"

    grep -Fq 'pull app' "${fixture}/calls" \
        || fail_test "--prepare did not pull the new app image"
    ! grep -Eq ' (run|up|stop|down|restart) ' "${fixture}/calls" \
        || fail_test "--prepare migrated or changed running services"
}

test_no_pull_still_migrates_before_recreate() {
    local fixture="${TEST_ROOT}/migration-without-pull"
    make_fixture "${fixture}"
    make_migration_docker_stub "${fixture}"

    PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" --no-pull \
        >"${fixture}/stdout" 2>"${fixture}/stderr"

    ! grep -Fq 'pull app' "${fixture}/calls" \
        || fail_test "--no-pull downloaded an app image"
    local migration_line recreate_line
    migration_line="$(grep -nF 'run --rm --no-deps -T app --migrate' "${fixture}/calls" | cut -d: -f1)"
    recreate_line="$(grep -nF 'up -d --wait --wait-timeout 120 app' "${fixture}/calls" | cut -d: -f1)"
    [[ "${migration_line}" -lt "${recreate_line}" ]] \
        || fail_test "--no-pull skipped schema preparation before recreate"
}

test_wait_failure_is_not_ignored() {
    local fixture="${TEST_ROOT}/wait-failure"
    make_fixture "${fixture}"
    cat >"${fixture}/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${AETHER_TEST_CALLS}"
case "$*" in
    "compose version"|"info") exit 0 ;;
    *"config --services") printf 'app\n' ;;
    *"up --help") printf '%s\n' '      --wait  Wait for services' ;;
    *"up -d --wait --wait-timeout 120 app") exit 42 ;;
    *"pull app") exit 0 ;;
esac
exit 0
EOF
    chmod 0755 "${fixture}/bin/docker"

    if PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" \
        >"${fixture}/stdout" 2>"${fixture}/stderr"; then
        fail_test "update succeeded after compose --wait reported an unhealthy app"
    fi

    [[ "$(grep -Fc 'up -d' "${fixture}/calls")" -eq 1 ]] \
        || fail_test "failed update retried an unverified simple recreate"
    ! grep -Fq '>>> Done.' "${fixture}/stdout" \
        || fail_test "failed update was reported as successful"
}

test_legacy_compose_uses_health_polling() {
    local fixture="${TEST_ROOT}/legacy-wait"
    make_fixture "${fixture}"
    cat >"${fixture}/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${AETHER_TEST_CALLS}"
case "$*" in
    "compose version"|"info") exit 0 ;;
    *"config --services") printf 'app\n' ;;
    *"up --help") printf '%s\n' 'Usage: docker compose up' ;;
    *"pull app"|*"up -d app"|*" ps") exit 0 ;;
    *"ps -q app") printf 'container-id\n' ;;
    "inspect --format={{.State.Health.Status}} container-id") printf 'healthy\n' ;;
esac
exit 0
EOF
    chmod 0755 "${fixture}/bin/docker"

    PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" \
        >"${fixture}/stdout" 2>"${fixture}/stderr"

    grep -Fq 'up -d app' "${fixture}/calls" \
        || fail_test "legacy Compose path did not recreate the app"
    grep -Fq '>>> Container is healthy.' "${fixture}/stdout" \
        || fail_test "legacy Compose path did not require a healthy container"
}

test_legacy_compose_health_failure_is_not_ignored() {
    local fixture="${TEST_ROOT}/legacy-unhealthy"
    make_fixture "${fixture}"
    cat >"${fixture}/bin/docker" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${AETHER_TEST_CALLS}"
case "$*" in
    "compose version"|"info") exit 0 ;;
    *"config --services") printf 'app\n' ;;
    *"up --help") printf '%s\n' 'Usage: docker compose up' ;;
    *"pull app"|*"up -d app") exit 0 ;;
    *"ps -q app") printf 'container-id\n' ;;
    "inspect --format={{.State.Health.Status}} container-id") printf 'unhealthy\n' ;;
esac
exit 0
EOF
    cat >"${fixture}/bin/sleep" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
    chmod 0755 "${fixture}/bin/docker" "${fixture}/bin/sleep"

    if PATH="${fixture}/bin:${PATH}" AETHER_TEST_CALLS="${fixture}/calls" \
        "${fixture}/compose/update.sh" --compose-dir "${fixture}/compose" \
        >"${fixture}/stdout" 2>"${fixture}/stderr"; then
        fail_test "legacy Compose update succeeded with an unhealthy app"
    fi

    grep -Fq 'health check timed out' "${fixture}/stdout" \
        || fail_test "legacy unhealthy app did not reach the health failure path"
    ! grep -Fq '>>> Done.' "${fixture}/stdout" \
        || fail_test "legacy unhealthy update was reported as successful"
}

test_option_like_service_name_is_rejected() {
    local fixture="${TEST_ROOT}/unsafe-service"
    make_fixture "${fixture}"

    if PATH="${fixture}/bin:${PATH}" "${fixture}/compose/update.sh" \
        --compose-dir "${fixture}/compose" --service --ansi \
        >"${fixture}/stdout" 2>"${fixture}/stderr"; then
        fail_test "option-like service name was accepted"
    fi
    grep -Fq 'service name contains unsafe characters' "${fixture}/stderr" \
        || fail_test "unsafe service name was not rejected by the identifier guard"
}

test_migration_succeeds_before_recreate
test_migration_failure_preserves_running_app
test_prepare_only_pulls_image
test_no_pull_still_migrates_before_recreate
test_wait_failure_is_not_ignored
test_legacy_compose_uses_health_polling
test_legacy_compose_health_failure_is_not_ignored
test_option_like_service_name_is_rejected

echo "PASS: compose updater migration preflight, failure, and argument safety fixtures"
