echo "| Test                          | Base         | PR               | % change   |\n" >> "benches/critcmp.txt"
echo "|-------------------------------|--------------|------------------|------------|\n" >> "benches/critcmp.txt"
critcmp new_branch main_branch | awk 'NR>2 {
    item = $1
    before = $3
    change = $5
    after = $7
    
    printf "| %-30s | %-20s | %-20s | %-10.2f |\n", item, before, after, change >> "benches/critcmp.txt"

    if (change > 1.1) {
         printf "Percentage change for %s exceeds 10%%.\n", item
         flag=1
    }
}

END {
    if (flag!=0) {
        print "CI failed due to exceeding percentage change."
        exit 1
    }
}'
