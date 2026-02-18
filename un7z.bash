#!/bin/bash

JOBS=3
FAILED_LOG="failed.log"
EXTRACT_LOG="extract.log"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

cleanup_interrupt() {
    echo -e "\n${YELLOW}Interrupted by user.${NC}"
    exit 130
}

trap cleanup_interrupt INT TERM

echo "Scanning archives (recursive)..."

archives=$(find . -type f \( \
-name "*.7z.001" -o \
-name "*.zip.001" -o \
-name "*.tar.gz" -o \
-name "*.tgz" -o \
-name "*.part*.rar" \
\) | sort -u)

archives=$(echo "$archives" | grep -E '\.7z\.001$|\.zip\.001$|\.tar\.gz$|\.tgz$|\.part0*1\.rar$')

if [ -z "$archives" ]; then
    echo "No archives found."
    exit 0
fi

# 用户自由选择解压项目，支持单选、范围选择和全选

archive_count=$(echo "$archives" | wc -l | tr -d ' ')
echo "Found $archive_count archives:"

archive_list=()
while IFS= read -r line; do
    archive_list+=("$line")
done <<< "$archives"
for i in "${!archive_list[@]}"; do
    printf "%3d) %s\n" $((i+1)) "${archive_list[$i]}"
done

read -r -p "Select archives (e.g. 1,3,5-7 or all) [all]: " SELECTION
if [ -z "$SELECTION" ] || [[ "$SELECTION" =~ ^[Aa][Ll][Ll]$ ]]; then
    selected_list=("${archive_list[@]}")
else
    selected_flags=()
    IFS=',' read -ra parts <<< "$SELECTION"
    for part in "${parts[@]}"; do
        part=$(echo "$part" | tr -d ' ')
        if [[ "$part" =~ ^[0-9]+$ ]]; then
            idx=$((part))
            if [ "$idx" -ge 1 ] && [ "$idx" -le "$archive_count" ]; then
                selected_flags[$idx]=1
            else
                echo -e "${RED}Invalid selection:${NC} $part"
                exit 1
            fi
        elif [[ "$part" =~ ^[0-9]+-[0-9]+$ ]]; then
            start=${part%-*}
            end=${part#*-}
            if [ "$start" -ge 1 ] && [ "$end" -le "$archive_count" ] && [ "$start" -le "$end" ]; then
                for ((idx=start; idx<=end; idx++)); do
                    selected_flags[$idx]=1
                done
            else
                echo -e "${RED}Invalid range:${NC} $part"
                exit 1
            fi
        else
            echo -e "${RED}Invalid selection:${NC} $part"
            exit 1
        fi
    done

    selected_list=()
    for ((idx=1; idx<=archive_count; idx++)); do
        if [ -n "${selected_flags[$idx]}" ]; then
            selected_list+=("${archive_list[$((idx-1))]}")
        fi
    done

    if [ "${#selected_list[@]}" -eq 0 ]; then
        echo -e "${RED}No valid archives selected.${NC}"
        exit 1
    fi
fi

> "$FAILED_LOG"

read -r -p "Run integrity test before extract? (y/N): " RUN_TEST
if [[ "$RUN_TEST" =~ ^[Yy]$ ]]; then
    DO_TEST=1
else
    DO_TEST=0
fi

read -s -p "Password (leave empty if none): " PASSWORD
echo ""

export PASSWORD FAILED_LOG EXTRACT_LOG RED GREEN YELLOW NC DO_TEST

log_msg() {
    echo -e "$1" >> "$EXTRACT_LOG"
}

extract_one() {

    archive="$1"
    name=$(basename "$archive")
    base="$name"

    if [[ "$base" =~ \.part[0-9]+\.rar$ ]]; then
        base="${base%%.part*}"
        TYPE="rar"
    elif [[ "$base" =~ \.7z\.001$ ]]; then
        base="${base%.7z.001}"
        TYPE="7z"
    elif [[ "$base" =~ \.zip\.001$ ]]; then
        base="${base%.zip.001}"
        TYPE="zip"
    elif [[ "$base" =~ \.tar\.gz$ ]] || [[ "$base" =~ \.tgz$ ]]; then
        base="${base%.tar.gz}"
        base="${base%.tgz}"
        TYPE="tar"
    else
        TYPE="unknown"
    fi

    if [ -d "$base" ]; then
        log_msg "${YELLOW}SKIP${NC} $archive"
        return 2
    fi

    mkdir -p "$base"

    if [ "$DO_TEST" -eq 1 ]; then
        log_msg "${GREEN}TEST${NC} $archive"

        if [ "$TYPE" = "rar" ]; then
            if [ -n "$PASSWORD" ]; then
                unrar t -p"$PASSWORD" "$archive" >> "$EXTRACT_LOG" 2>&1
            else
                unrar t "$archive" >> "$EXTRACT_LOG" 2>&1
            fi
        elif [ "$TYPE" = "7z" ] || [ "$TYPE" = "zip" ]; then
            if [ -n "$PASSWORD" ]; then
                7zz t -p"$PASSWORD" "$archive" >> "$EXTRACT_LOG" 2>&1
            else
                7zz t "$archive" >> "$EXTRACT_LOG" 2>&1
            fi
        elif [ "$TYPE" = "tar" ]; then
            tar -tzf "$archive" >> "$EXTRACT_LOG" 2>&1
        fi

        if [ $? -ne 0 ]; then
            log_msg "${RED}CRC FAIL${NC} $archive"
            echo "$archive" >> "$FAILED_LOG"
            rm -rf "$base"
            return 3
        fi
    fi

    log_msg "${GREEN}EXTRACT${NC} $archive"

    if [ "$TYPE" = "rar" ]; then
        if [ -n "$PASSWORD" ]; then
            unrar x -o+ -p"$PASSWORD" "$archive" "$base/" >> "$EXTRACT_LOG" 2>&1
        else
            unrar x -o+ "$archive" "$base/" >> "$EXTRACT_LOG" 2>&1
        fi
    elif [ "$TYPE" = "7z" ] || [ "$TYPE" = "zip" ]; then
        if [ -n "$PASSWORD" ]; then
            7zz x -p"$PASSWORD" "$archive" -o"$base" -y >> "$EXTRACT_LOG" 2>&1
        else
            7zz x "$archive" -o"$base" -y >> "$EXTRACT_LOG" 2>&1
        fi
    elif [ "$TYPE" = "tar" ]; then
        tar -xzf "$archive" -C "$base"
    fi

    if [ $? -ne 0 ]; then
        log_msg "${RED}FAILED${NC} $archive"
        echo "$archive" >> "$FAILED_LOG"
        rm -rf "$base"
        return 4
    fi

    log_msg "${GREEN}DONE${NC} $archive"
    return 0
}

export -f extract_one

render_progress() {
    local done="$1"
    local total="$2"
    local label="$3"
    local width=30
    local filled=$((done * width / total))
    local empty=$((width - filled))
    local percent=$((done * 100 / total))
    local bar_filled
    local bar_empty
    local max_label=40
    if [ ${#label} -gt $max_label ]; then
        label="${label:0:$((max_label-1))}…"
    fi
    bar_filled=$(printf "%*s" "$filled" "" | tr ' ' '=')
    if [ "$empty" -gt 0 ]; then
        bar_empty=$(printf "%*s" "$empty" "" | tr ' ' '.')
    else
        bar_empty=""
    fi
    printf "\r\033[K[%s%s] %3d%% %d/%d %s" "$bar_filled" "$bar_empty" "$percent" "$done" "$total" "$label"
}

total_selected=${#selected_list[@]}
done_count=0

for archive in "${selected_list[@]}"; do
    short_label=$(basename "$archive")
    render_progress "$done_count" "$total_selected" "Extracting: $short_label"
    extract_one "$archive"
    status=$?
    done_count=$((done_count + 1))
    if [ "$status" -eq 0 ]; then
        render_progress "$done_count" "$total_selected" "Done: $short_label"
    elif [ "$status" -eq 2 ]; then
        render_progress "$done_count" "$total_selected" "Skipped: $short_label"
    else
        render_progress "$done_count" "$total_selected" "Failed: $short_label"
    fi
done
printf "\n"

if [ -s "$FAILED_LOG" ]; then
    echo -e "${RED}Some archives failed:${NC}"
    cat "$FAILED_LOG"
else
    echo -e "${GREEN}All archives processed successfully.${NC}"
fi

