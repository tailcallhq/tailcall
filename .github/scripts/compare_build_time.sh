#!/bin/bash
file1="benches/iai-callgrind/old_benchmark.txt"
file2="benches/iai-callgrind/new_benchmark.txt"
config_file="benches/iai-callgrind/benchmarks.cfg"
readarray -t benchmarks < "$config_file"
attributes=("Instructions" "L1 Hits" "L2 Hits" "RAM Hits" "Total read+write" "Estimated Cycles")
fail_ci=0

calculate_value() {
    local file="$1"
    local bench="$2"
    local attribute="$3"
    case "$attribute" in
        "Total read+write")
            echo $(( $(grep -A5 "$bench" "$file" | grep -Po "L1 Hits:\s*\K\d+" || echo 0) +
                     $(grep -A5 "$bench" "$file" | grep -Po "L2 Hits:\s*\K\d+" || echo 0) +
                     $(grep -A5 "$bench" "$file" | grep -Po "RAM Hits:\s*\K\d+" || echo 0) ))
            ;;
        "Estimated Cycles")
            echo $(( $(grep -A5 "$bench" "$file" | grep -Po "L1 Hits:\s*\K\d+" || echo 0) +
                     5 * $(grep -A5 "$bench" "$file" | grep -Po "L2 Hits:\s*\K\d+" || echo 0) +
                     35 * $(grep -A5 "$bench" "$file" | grep -Po "RAM Hits:\s*\K\d+" || echo 0) ))
            ;;
        *)
            echo $(grep -A5 "$bench" "$file" | grep -Po "${attribute}:\s*\K\d+" || echo 0)
            ;;
    esac
}

for bench in "${benchmarks[@]}"; do
    for attribute in "${attributes[@]}"; do
        value1=$(calculate_value "$file1" "$bench" "$attribute")
        value2=$(calculate_value "$file2" "$bench" "$attribute")

        percent_change=$(( value1 ? ((value2 - value1) * 100) / value1 : 0 ))

        if ((percent_change > 10)); then
            echo "$bench $attribute has a change of $percent_change%, failing CI. (Original values: $value1 -> $value2)"
            fail_ci=1
        else
            echo "$bench $attribute has a change of $percent_change%, within CI limits. (Original values: $value1 -> $value2)"
        fi
    done
    echo "----------------------------------"
done

exit "$fail_ci"
