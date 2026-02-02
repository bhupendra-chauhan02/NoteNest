#!/usr/bin/env bash
set -euo pipefail

base_url="https://princey9.github.io/NoteNest"

declare -a urls=(
  "$base_url/"
  "$base_url/pricing/"
  "$base_url/demo/"
)

declare -a files=(
  "docs/index.html"
  "docs/pricing/index.html"
  "docs/demo/index.html"
  "docs/assets/app.js"
  "docs/assets/styles.css"
  "docs/favicon.svg"
)

fail_count=0

print_result() {
  local label="$1"
  local status="$2"
  printf "%-45s %s\n" "$label" "$status"
}

check_file() {
  local path="$1"
  if [[ -f "$path" ]]; then
    print_result "$path" "PASS"
  else
    print_result "$path" "FAIL"
    fail_count=$((fail_count + 1))
  fi
}

check_url() {
  local label="$1"
  local url="$2"
  local expected="$3"
  local code

  code=$(curl -s -L -o /dev/null -w "%{http_code}" "$url" || echo "000")
  if echo "$expected" | grep -q "$code"; then
    print_result "$label" "PASS ($code)"
  else
    print_result "$label" "FAIL ($code)"
    fail_count=$((fail_count + 1))
  fi
}

check_contains() {
  local path="$1"
  local pattern="$2"
  local label="$3"

  if rg -q "$pattern" "$path"; then
    print_result "$label" "PASS"
  else
    print_result "$label" "FAIL"
    fail_count=$((fail_count + 1))
  fi
}

echo "Site Doctor"
print_result "Check" "Result"

echo ""
print_result "HTTP checks" ""
for url in "${urls[@]}"; do
  check_url "$url" "$url" "200"
done

echo ""
print_result "Local files" ""
for file in "${files[@]}"; do
  check_file "$file"
done

echo ""
print_result "Nav links" ""
check_contains "docs/index.html" 'href="\./"' "docs/index.html home"
check_contains "docs/index.html" 'href="pricing/"' "docs/index.html pricing"
check_contains "docs/index.html" 'href="demo/"' "docs/index.html demo"
check_contains "docs/index.html" 'href="https://github.com/Princey9/NoteNest"' "docs/index.html github"

check_contains "docs/pricing/index.html" 'href="\.\./"' "docs/pricing/index.html home"
check_contains "docs/pricing/index.html" 'href="\.\./pricing/"' "docs/pricing/index.html pricing"
check_contains "docs/pricing/index.html" 'href="\.\./demo/"' "docs/pricing/index.html demo"
check_contains "docs/pricing/index.html" 'href="https://github.com/Princey9/NoteNest"' "docs/pricing/index.html github"

check_contains "docs/demo/index.html" 'href="\.\./"' "docs/demo/index.html home"
check_contains "docs/demo/index.html" 'href="\.\./pricing/"' "docs/demo/index.html pricing"
check_contains "docs/demo/index.html" 'href="\.\./demo/"' "docs/demo/index.html demo"
check_contains "docs/demo/index.html" 'href="https://github.com/Princey9/NoteNest"' "docs/demo/index.html github"

echo ""
print_result "Assets referenced" ""
check_contains "docs/index.html" 'assets/styles.css' "docs/index.html styles"
check_contains "docs/index.html" 'assets/app.js' "docs/index.html app"
check_contains "docs/index.html" 'favicon.svg' "docs/index.html favicon"

check_contains "docs/pricing/index.html" '../assets/styles.css' "docs/pricing/index.html styles"
check_contains "docs/pricing/index.html" '../assets/app.js' "docs/pricing/index.html app"
check_contains "docs/pricing/index.html" '../favicon.svg' "docs/pricing/index.html favicon"

check_contains "docs/demo/index.html" '../assets/styles.css' "docs/demo/index.html styles"
check_contains "docs/demo/index.html" '../assets/app.js' "docs/demo/index.html app"
check_contains "docs/demo/index.html" '../favicon.svg' "docs/demo/index.html favicon"

echo ""
print_result "Download links" ""
check_contains docs/index.html 'releases/latest' "releases/latest link"
check_contains docs/index.html 'releases/latest/download/' "direct asset links"

if [[ $fail_count -eq 0 ]]; then
  echo ""
  print_result "Overall" "PASS"
  exit 0
fi

echo ""
print_result "Overall" "FAIL"
exit 1
