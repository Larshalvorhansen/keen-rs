#!/usr/bin/env bash

# Minsky upstream build diagnostics
# Read-only with respect to the upstream source tree.
# Does not execute Make or install dependencies.

REPO="$HOME/Documents/keen-rs/minsky"
OUT="$HOME/Documents/keen-rs/minsky-diagnostics"
ARCHIVE="$HOME/Documents/keen-rs/minsky-diagnostics.tar.gz"

mkdir -p "$OUT"
cd "$REPO" || exit 1

echo "========================================"
echo "MINSKY UPSTREAM BUILD DIAGNOSTICS"
echo "========================================"

echo "[1/10] System and Nix environment"

{
  date
  uname -a
  uname -m
  sw_vers 2>/dev/null || true
  echo "IN_NIX_SHELL=${IN_NIX_SHELL:-not set}"
  echo "SDKROOT=${SDKROOT:-not set}"
  echo "MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-not set}"
  echo "CC=${CC:-not set}"
  echo "CXX=${CXX:-not set}"
  echo "PATH=$PATH"
} > "$OUT/system.txt"

echo "[2/10] Compiler information"

{
  for cmd in cc c++ clang clang++ gcc g++ make cmake \
             pkg-config python3 bison flex perl git; do
    echo "=== $cmd ==="
    command -v "$cmd" || true
    case "$cmd" in
      cc|c++|clang|clang++|gcc|g++)
        "$cmd" --version 2>&1 | head -3 ;;
      python3)
        "$cmd" --version 2>&1 ;;
    esac
  done
} > "$OUT/compilers.txt" 2>&1

echo "[3/10] Library availability"

{
  for lib in \
    gsl \
    cairo \
    pango \
    pangocairo \
    librsvg-2.0 \
    freetype2 \
    fontconfig \
    libpng \
    libxml-2.0 \
    zlib \
    openssl \
    sqlite3 \
    libpq \
    python3 \
    tcl \
    tk; do

    printf "%-22s " "$lib"

    if pkg-config --exists "$lib" 2>/dev/null; then
      pkg-config --modversion "$lib"
      pkg-config --cflags --libs "$lib"
    else
      echo "NOT FOUND by pkg-config"
    fi
  done
} > "$OUT/libraries.txt" 2>&1

echo "[4/10] Boost and compiler smoke tests"

{
  echo "=== C++ standard library ==="
  c++ -v 2>&1 | tail -20

  echo "=== Boost headers and linker ==="

  cat > "$OUT/boost_probe.cc" <<'EOF'
#include <boost/version.hpp>
#include <boost/filesystem.hpp>
#include <iostream>

int main() {
  std::cout << BOOST_LIB_VERSION << "\n";
  boost::filesystem::path p("test");
  std::cout << p.string() << "\n";
}
EOF

  c++ -std=c++17 "$OUT/boost_probe.cc" \
    -o "$OUT/boost_probe" \
    -lboost_filesystem 2>&1

  if [ -x "$OUT/boost_probe" ]; then
    "$OUT/boost_probe"
    echo "BOOST_COMPILE_LINK=OK"
  else
    echo "BOOST_COMPILE_LINK=FAILED"
  fi
} > "$OUT/boost.txt" 2>&1

echo "[5/10] Classdesc status"

{
  for f in \
    ecolab/bin/classdesc \
    ecolab/classdesc/classdesc \
    ecolab/classdesc/Makefile \
    ecolab/include/Makefile.config \
    ecolab/include/TCL_obj_base.cd; do

    echo "=== $f ==="
    ls -l "$f" 2>&1 || true
    file "$f" 2>&1 || true

    if [ -x "$f" ]; then
      echo "EXECUTABLE=YES"
    fi
  done

  echo "=== Related tools ==="
  ls -l ecolab/bin/ 2>&1 | head -50
} > "$OUT/classdesc.txt" 2>&1

echo "[6/10] Ecolab configuration"

{
  echo "=== Generated configuration ==="
  cat ecolab/include/Makefile.config 2>&1

  echo
  echo "=== Build configuration ==="
  grep -nE 'boost|BOOST|CXX|GCC|CLANG|CLASSDESC|PYTHON|AQUA|DARWIN|Darwin' \
    ecolab/include/Makefile | head -110

  echo
  echo "=== Ecolab Makefile ==="
  grep -nE 'boost|BOOST|CXX|GCC|CLANG|classdesc|PYTHON|Darwin' \
    ecolab/Makefile | head -90
} > "$OUT/ecolab-config.txt" 2>&1

echo "[7/10] Minsky Makefile analysis"

{
  echo "=== Build variables ==="
  grep -nE 'PYMINSKY|PYTHON|CXX|BOOST|GSL|LIBS|FLAGS|MACOSX|ARCH|^OS|Darwin' \
    Makefile | head -180

  echo
  echo "=== Python extension rules ==="
  sed -n '450,495p' Makefile

  echo
  echo "=== Top-level dependency rules ==="
  sed -n '1,165p' Makefile
} > "$OUT/minsky-makefile.txt" 2>&1

echo "[8/10] Python binding diagnostics"

{
  echo "=== Python version ==="
  python3 --version
  command -v python3

  echo
  echo "=== Python configuration ==="
  python3 -c '
import sys, sysconfig
print("Executable:", sys.executable)
print("Version:", sys.version)
print("Include:", sysconfig.get_path("include"))
print("Extension suffix:", sysconfig.get_config_var("EXT_SUFFIX"))
print("Platform:", sysconfig.get_platform())
'

  echo
  echo "=== Existing extension files ==="
  find . -maxdepth 3 -name 'pyminsky*.so' -print

  echo
  echo "=== Import test ==="
  PYTHONPATH="$PWD" python3 -c \
    'from pyminsky import minsky; print("IMPORT OK")' 2>&1

  echo
  echo "=== Python binding source ==="
  ls -lh RESTService/pyminsky.cc \
    ecolab/classdesc/pythonCAPI.cc 2>&1
} > "$OUT/python.txt" 2>&1

echo "[9/10] Git and build state"

{
  echo "=== Upstream revision ==="
  git rev-parse HEAD
  git submodule status --recursive

  echo
  echo "=== Working tree changes (first 80) ==="
  git status --short | head -80

  echo
  echo "=== Existing generated files ==="
  for f in \
    pyminsky.o \
    pyminsky.so \
    libminsky.a \
    ecolab/lib/libecolab.a \
    ecolab/build.log; do
    ls -lh "$f" 2>&1 || true
  done
} > "$OUT/git-and-build.txt" 2>&1

echo "[10/10] Previous build errors"

{
  for log in ecolab/build.log build.log; do
    if [ -f "$log" ]; then
      echo "=== $log ==="
      grep -nE \
        'error:|Error |cannot figure|not found|No such file|undefined symbol|ld:' \
        "$log" | tail -100
    fi
  done
} > "$OUT/errors.txt" 2>&1

echo "=== Creating diagnostic summary ==="

{
  echo "MINSKY BUILD DIAGNOSTIC SUMMARY"
  echo "================================"
  echo
  echo "System:"
  uname -m

  echo
  echo "Python:"
  python3 --version

  echo
  echo "Classdesc:"
  if [ -x ecolab/bin/classdesc ]; then
    echo "Executable available"
  else
    echo "Executable unavailable"
  fi

  echo
  echo "Boost:"
  grep 'BOOST_COMPILE_LINK=' "$OUT/boost.txt"

  echo
  echo "Python extension:"
  if [ -f pyminsky.so ]; then
    echo "Built"
  else
    echo "Not built"
  fi

  echo
  echo "Package availability:"
  grep 'NOT FOUND' "$OUT/libraries.txt"

  echo
  echo "Previous build errors:"
  tail -25 "$OUT/errors.txt"
} > "$OUT/SUMMARY.txt"

cd "$HOME/Documents/keen-rs" || exit 1

tar -czf "$ARCHIVE" minsky-diagnostics

echo
echo "========================================"
echo "DIAGNOSTICS COMPLETE"
echo "========================================"
echo
cat "$OUT/SUMMARY.txt"

echo
echo "Archive: $ARCHIVE"
du -h "$ARCHIVE"
echo
echo "Upload minsky-diagnostics.tar.gz to ChatGPT."
