current_branch=$(git rev-parse --abbrev-ref HEAD) 
critcmp new_branch main_branch | awk 'NR>2 {
    item = $1
    before = $3
    after = $7
    before_val = ($3 ~ /ns/) ? $3 : ($3 ~ /µs/) ? $3 * 1000 : ($3 ~ /ms/) ? $3 * 1000000 : "invalid"
    after_val = ($7 ~ /ns/) ? $7 : ($7 ~ /µs/) ? $7 * 1000 : ($7 ~ /ms/) ? $7 * 1000000 : "invalid"

    if (before_val != "invalid" && after_val != "invalid") {
        change = ((after_val - before_val) / before_val) * 100
        gsub("%", "", change)  # Remove '%' symbol

        printf "| %-30s | %-20s | %-20s | %-10.2f |\n", item, before, after, change >> "benches/critcmp.txt"

        if (change > 10) {
            printf "Percentage change for %s exceeds 10%%. Failing the workflow.\n", item
            exit 1
        }
    } else {
        printf "Invalid units detected for %s. Failing the workflow.\n", item
        exit 1
    }
}'
