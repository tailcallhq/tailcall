echo "critcmp main_branch new_branch"
critcmp main_branch new_branch | awk 'NR>2 {
    item = $1
    before = $7
    after = $3
    before_val = ($7 ~ /ns/) ? $7 : ($7 ~ /µs/) ? $7 * 1000 : ($7 ~ /ms/) ? $7 * 1000000 : "invalid"
    after_val = ($3 ~ /ns/) ? $3 : ($3 ~ /µs/) ? $3 * 1000 : ($3 ~ /ms/) ? $3 * 1000000 : "invalid"

    temp1 = before_val
    temp2 = after_val

    if (before_val != "invalid" && after_val != "invalid") {
        change = ((after_val - before_val) / before_val) * 100
        gsub("%", "", change)  # Remove '%' symbol

        printf "| %-30s | %-20s | %-20s | %-10.2f |\n", item, before, after, change >> "output_file.txt"

        if (change > 10) {
            echo "Percentage change exceeds 10%. Failing the workflow."
            exit 1
        }
    } else {
        echo "Invalid units detected for %s. Failing the workflow."
        exit 1
    }
}'
