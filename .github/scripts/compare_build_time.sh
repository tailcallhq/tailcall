#!/bin/bash

# Run benchmarks and save output to another file
echo -n > benches/iai-callgrind/new_benchmarks.txt
cargo bench --bench json_like_bench_iai-callgrind -- --save-baseline change >> benches/iai-callgrind/new_benchmarks.txt
cargo bench --bench data_loader_bench_iai-callgrind -- --save-baseline change >> benches/iai-callgrind/new_benchmarks.txt
cargo bench --bench impl_path_string_for_evaluation_context_iai-callgrind -- --save-baseline change >> benches/iai-callgrind/new_benchmarks.txt
cargo bench --bench request_template_bench_iai-callgrind -- --save-baseline change >> benches/iai-callgrind/new_benchmarks.txt
sed -i 's/ \{1,\}\([0-9]\)/\1/g' benches/iai-callgrind/new_benchmarks.txt
file2="benches/iai-callgrind/new_benchmarks.txt"

# Switch to main branch
git fetch
git checkout main

# Run benchmarks and save output to a file
echo -n > benches/iai-callgrind/old_benchmark.txt
cargo bench --bench json_like_bench_iai-callgrind -- --save-baseline main >> benches/iai-callgrind/old_benchmark.txt
cargo bench --bench data_loader_bench_iai-callgrind -- --save-baseline main >> benches/iai-callgrind/old_benchmark.txt
cargo bench --bench impl_path_string_for_evaluation_context_iai-callgrind -- --save-baseline main >> benches/iai-callgrind/old_benchmark.txt
cargo bench --bench request_template_bench_iai-callgrind -- --save-baseline main >> benches/iai-callgrind/old_benchmark.txt
sed -i 's/ \{1,\}\([0-9]\)/\1/g' benches/iai-callgrind/old_benchmark.txt


# Switch to current branch
file1="benches/iai-callgrind/old_benchmark.txt"

config_file="benches/iai-callgrind/benchmarks.cfg" # to add more benchmarks add in this file

# Read benchmarks from the configuration file
readarray -t benchmarks < "$config_file"

attributes=("Instructions" "L1 Hits" "L2 Hits" "RAM Hits")

fail_ci=0

for bench in "${benchmarks[@]}"; do
    x=0
    y=0

    # Check attribute changes
    for attribute in "${attributes[@]}"; do
        value1=$(grep -A5 "$bench" "$file1" | grep -Po "${attribute}:\K\d+")
        value2=$(grep -A5 "$bench" "$file2" | grep -Po "${attribute}:\K\d+")

        if [ -n "$value1" ] && [ -n "$value2" ]; then
            if [ "$value1" -ne 0 ]; then
                percent_change=$(( ((value2 - value1) * 100) / value1 ))
            else
                percent_change="nan"
            fi

            if ((percent_change > 10)); then
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
    # Total read+write = L1 Hits + L2 Hits + RAM Hits.
    # Estimated Cycles = L1 Hits + 5 × (L2 Hits) + 35 × (RAM Hits).
    for file in "$file1" "$file2"; do
        l1_hits=$(grep -A5 "$bench" "$file" | grep -Po "L1 Hits:\K\d+")
        l2_hits=$(grep -A5 "$bench" "$file" | grep -Po "L2 Hits:\K\d+")
        ram_hits=$(grep -A5 "$bench" "$file" | grep -Po "RAM Hits:\K\d+")

        if [ "$x" -ne 0 ]; then
            total_read_write=$((l1_hits + l2_hits + ram_hits))
            estimated_cycles=$((l1_hits + 5 * l2_hits + 35 * ram_hits))

            p1=$(( ((total_read_write - x) * 100) / x ))
            echo "$bench Total read+write has a change of $p1%"

            if ((p1 > 10)); then
                echo "$bench Total read+write has a change greater than 10%, failing CI."
                fail_ci=1
            fi

            p2=$(( ((estimated_cycles - y) * 100) / y ))
            echo "$bench Estimated Cycles has a change of $p2%"

            if ((p2 > 10)); then
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

exit "$fail_ci"
