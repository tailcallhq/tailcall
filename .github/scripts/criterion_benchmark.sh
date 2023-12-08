echo "| Test                           | Base         | PR           | % change   |" 
echo "|--------------------------------|--------------|--------------|------------|" 
fail_ci=0
critcmp main_branch new_branch | awk 'NR>2 {
    item = $1
    before = $3
    change = $6
    after = $7
    
    printf "| %-30s | %-20s | %-20s | %-10.2f |\n", item, before, after, (change-1.00)*100

    if (change > 1.1) {
         printf "Percentage change for %s exceeds 10%%.\n", item >> "benches/critcmp.txt"
         fail_ci=1
    }
}'

if [ "$fail_ci" -eq 1 ]; then
    echo "$(cat benches/critcmp.txt)"
    exit 1
fi
