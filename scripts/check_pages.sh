#!/usr/bin/env bash
set -euo pipefail

check_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Missing required file: $path" >&2
    exit 1
  fi
}

check_file "docs/index.html"
check_file "docs/assets/styles.css"
check_file "docs/assets/app.js"
check_file "docs/pricing/index.html"
check_file "docs/demo/index.html"

python3 -m http.server 8001 --directory docs >/tmp/notenest_pages.log 2>&1 &
server_pid=$!
sleep 1
root_code=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8001/)
demo_code=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:8001/demo/)
kill "$server_pid" 2>/dev/null || true

if [[ "$root_code" != "200" ]]; then
  echo "Homepage did not return 200 (got $root_code)" >&2
  exit 1
fi

if [[ "$demo_code" != "200" ]]; then
  echo "Demo page did not return 200 (got $demo_code)" >&2
  exit 1
fi

for page in docs/index.html docs/pricing/index.html docs/demo/index.html; do
  grep -q "Home" "$page" || { echo "Navbar missing Home in $page" >&2; exit 1; }
  grep -q "Pricing" "$page" || { echo "Navbar missing Pricing in $page" >&2; exit 1; }
  grep -q "Demo" "$page" || { echo "Navbar missing Demo in $page" >&2; exit 1; }
  grep -q "GitHub" "$page" || { echo "Navbar missing GitHub in $page" >&2; exit 1; }
  done

grep -q "Convert" docs/demo/index.html || { echo "Demo page missing Convert button" >&2; exit 1; }
grep -q "Clear" docs/demo/index.html || { echo "Demo page missing Clear button" >&2; exit 1; }
grep -q "Summarize &amp; Structure" docs/demo/index.html || { echo "Demo page missing Summarize mode" >&2; exit 1; }
grep -q "Cloak Mode" docs/demo/index.html || { echo "Demo page missing Cloak mode" >&2; exit 1; }

grep -q "Patient view" docs/demo/index.html || { echo "Demo page missing Patient view tab" >&2; exit 1; }

grep -q "Clinician view" docs/demo/index.html || { echo "Demo page missing Clinician view tab" >&2; exit 1; }

echo "Docs smoke checks passed."
