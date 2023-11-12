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
            for attribute in "${attributes[@]}"; do
                value1=$(grep -A5 "$bench" "file1.txt" | grep -Po "${attribute}:\K\d+")
                value2=$(grep -A5 "$bench" "$file2" | grep -Po "${attribute}:\K\d+")

                if [ -n "$value1" ] && [ -n "$value2" ] && [ "$attribute" != "Total read+write" ] && [ "$attribute" != "Estimated Cycles" ]; then
                    percent_change=$(awk -v v1="$value1" -v v2="$value2" 'BEGIN { pc = (v2 - v1) / v1 * 100; printf "%.2f", pc }')
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
        done

        exit $fail_ci
#end