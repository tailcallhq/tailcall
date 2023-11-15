#!/bin/bash

file1_url="https://raw.githubusercontent.com/alankritdabral/tailcall/main/benches/iai-callgrind/benchmarks.txt"
file2="benches/iai-callgrind/benchmarks.txt"

# Fetching file1 from the specified URL
curl -s "$file1_url" > file1.txt || { echo "Failed to download file from $file1_url"; exit 1; }

benchmarks=(
    "json_like_bench_iai_callgrind::batched_body::benchmark_batched_body"
    "data_loader_bench_iai_callgrind::data_loader::benchmark_data_loader"
    "impl_path_string_for_evaluation_context_iai_callgrind::bench::bench_main"
    "request_template_bench_iai_callgrind::bench_to_request::benchmark_to_request"
    # Add more benchmarks here as needed
)

attributes=("Instructions" "L1 Hits" "L2 Hits" "RAM Hits")

fail_ci=0

for bench in "${benchmarks[@]}"; do
    x=0
    y=0

    # Check attribute changes
    for attribute in "${attributes[@]}"; do
        value1=$(grep -A5 "$bench" "file1.txt" | grep -Po "${attribute}:\K\d+")
        value2=$(grep -A5 "$bench" "$file2" | grep -Po "${attribute}:\K\d+")

        if [ -n "$value1" ] && [ -n "$value2" ]; then
            percent_change=$(awk -v v1="$value1" -v v2="$value2" 'BEGIN { if(v1 != 0) pc = (v2 - v1) / v1 * 100; else pc = "nan"; printf "%.2f", pc }')
            if (( $(awk -v pc="$percent_change" 'BEGIN { print (pc > 10) }') )); then
                echo "$bench $attribute has a change of $percent_change%, failing CI."
                fail_ci=1
            else
                echo "$bench $attribute has a change of $percent_change%, within CI limits."
            fi
        else
            echo "Values not found for $bench $attribute"
        fi
    done

    # Check performance metric changes 
    #Total read+write = L1 Hits + L2 Hits + RAM Hits.
    #Estimated Cycles = L1 Hits + 5 × (L2 Hits) + 35 × (RAM Hits)
    for file in "file1.txt" "$file2"; do
        l1_hits=$(grep -A5 "$bench" "$file" | grep -Po "L1 Hits:\K\d+")
        l2_hits=$(grep -A5 "$bench" "$file" | grep -Po "L2 Hits:\K\d+")
        ram_hits=$(grep -A5 "$bench" "$file" | grep -Po "RAM Hits:\K\d+")

        if [ $x -ne 0 ]; then
            p1=$(( ( (l1_hits + l2_hits + ram_hits) - x ) * 100 / x ))
            echo "$bench Total read+write has a change of $p1%"

            if (( p1 > 10 )); then
                echo "$bench Total read+write has a change greater than 10%, failing CI."
                fail_ci=1
            fi

            p2=$(( ( (l1_hits + 5 * l2_hits + 35 * ram_hits) - y ) * 100 / y ))
            echo "$bench Estimated Cycles has a change of $p2%"

            if (( p2 > 10 )); then
                echo "$bench Estimated Cycles has a change greater than 10%, failing CI."
                fail_ci=1
            fi
        else
            total_read_write=$((l1_hits + l2_hits + ram_hits))
            estimated_cycles=$((l1_hits + 5 * l2_hits + 35 * ram_hits))
        fi

        x=$((x + total_read_write))
        y=$((y + estimated_cycles))
    done

    echo "----------------------------------"
done

exit $fail_ci
