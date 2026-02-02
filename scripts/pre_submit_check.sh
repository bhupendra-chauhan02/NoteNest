#!/usr/bin/env bash
set -euo pipefail

scripts/site_doctor.sh
scripts/release_doctor.sh

echo "ALL CHECKS PASSED"
