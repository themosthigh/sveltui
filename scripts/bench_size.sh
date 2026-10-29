#!/usr/bin/env bash
set -euo pipefail

# Ensure we run from workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

TARGETS=("hello_world" "counter" "component_form")
PROFILES=("debug" "release" "release-small")

format_bytes() {
    local bytes=$1
    if [ "$bytes" -ge 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f MB\", $bytes/1048576}")"
    elif [ "$bytes" -ge 1024 ]; then
        echo "$(awk "BEGIN {printf \"%.2f KB\", $bytes/1024}")"
    else
        echo "${bytes} B"
    fi
}

get_file_size() {
    local file=$1
    stat -f%z "$file" 2>/dev/null || stat -c%s "$file"
}

echo ""
echo "=========================================================================="
echo "                    Sveltui Binary Size Benchmarks                        "
echo "=========================================================================="
printf "%-20s %-16s %-14s %-14s\n" "Binary Target" "Profile" "Unstripped" "Stripped"
printf "%s\n" "--------------------------------------------------------------------------"

for target in "${TARGETS[@]}"; do
    for profile in "${PROFILES[@]}"; do
        flag=""
        bin_dir="target/debug"
        if [ "$profile" == "release" ]; then
            flag="--release"
            bin_dir="target/release"
        elif [ "$profile" == "release-small" ]; then
            flag="--profile release-small"
            bin_dir="target/release-small"
        fi

        # Build example binary
        cargo build --example "$target" $flag --quiet

        bin_path="$bin_dir/examples/$target"
        if [ ! -f "$bin_path" ]; then
            echo "Error: binary not found at $bin_path"
            exit 1
        fi

        raw_size=$(get_file_size "$bin_path")
        
        # Strip temporary copy to get stripped binary footprint
        tmp_bin=$(mktemp)
        cp "$bin_path" "$tmp_bin"
        strip "$tmp_bin" 2>/dev/null || true
        stripped_size=$(get_file_size "$tmp_bin")
        rm -f "$tmp_bin"

        printf "%-20s %-16s %-14s %-14s\n" \
            "$target" \
            "$profile" \
            "$(format_bytes "$raw_size")" \
            "$(format_bytes "$stripped_size")"
    done
    printf "%s\n" "--------------------------------------------------------------------------"
done

echo ""
