#!/usr/bin/env bash
# DEPRECATED — superseded by verify_html5_pipeline.sh (Cook 7, Stage 6 PASS)
# This script used genie_server.js + genie-web/ which no longer exist.
# Use the proven pipeline instead:
#
#   ./package-brm-html5.sh          # cook + package (Cook 7 flags)
#   ./verify_html5_pipeline.sh      # serve + Playwright + receipt PASS
#
# Or via rocket CLI:
#   ./rocket html5 cook --project Brm
#   ./rocket html5 serve --port 8080
#
echo "ERROR: stage6-serve-and-test.sh is deprecated. Use ./verify_html5_pipeline.sh instead." >&2
exit 1
