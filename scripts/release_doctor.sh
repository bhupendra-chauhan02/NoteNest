#!/usr/bin/env bash
set -euo pipefail

base_repo="https://github.com/Princey9/NoteNest"
release_url="$base_repo/releases/latest"

declare -a assets=(
  "aarch64-unknown-linux-musl.tar.gz"
  "x86_64-unknown-linux-musl.tar.gz"
)

fail_count=0

print_result() {
  local label="$1"
  local status="$2"
  printf "%-55s %s\n" "$label" "$status"
}

check_url_code() {
  local label="$1"
  local url="$2"
  local expected="$3"
  local code

  code=$(curl -s -I -L -o /dev/null -w "%{http_code}" "$url" || echo "000")
  if echo "$expected" | grep -q "$code"; then
    print_result "$label" "PASS ($code)"
  else
    print_result "$label" "FAIL ($code)"
    fail_count=$((fail_count + 1))
  fi
}

echo "Release Doctor"
print_result "Check" "Result"

check_url_code "$release_url" "$release_url" "200|302"

for asset in "${assets[@]}"; do
  asset_url="$base_repo/releases/latest/download/$asset"
  check_url_code "$asset" "$asset_url" "200"
done

if [[ $fail_count -eq 0 ]]; then
  echo ""
  print_result "Overall" "PASS"
  exit 0
fi

echo ""
print_result "Overall" "FAIL"
exit 1
