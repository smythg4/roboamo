#!/bin/bash

echo "Benchmarking with database updates..."
> times_with_update.txt
for i in {1..100}; do
    ./target/release/roboamo -u 2>&1 | \
    grep "Completion Time (Manning Allocation):" | \
    grep -oE '[0-9]+\.[0-9]+' >> times_with_update.txt
done

echo "Benchmarking without database updates..."
> times_without_update.txt
for i in {1..100}; do
    ./target/release/roboamo 2>&1 | \
    grep "Completion Time (Manning Allocation):" | \
    grep -oE '[0-9]+\.[0-9]+' >> times_without_update.txt
done

echo -e "\n=== WITH Database Updates ==="
awk '{
    sum+=$1; 
    count++; 
    if(NR==1 || $1<min) min=$1; 
    if(NR==1 || $1>max) max=$1;
} 
END {
    printf "Runs: %d\n", count;
    printf "Min: %.3fms\n", min;
    printf "Max: %.3fms\n", max;
    printf "Avg: %.3fms\n", sum/count;
}' times_with_update.txt

echo -e "\n=== WITHOUT Database Updates ==="
awk '{
    sum+=$1; 
    count++; 
    if(NR==1 || $1<min) min=$1; 
    if(NR==1 || $1>max) max=$1;
} 
END {
    printf "Runs: %d\n", count;
    printf "Min: %.3fms\n", min;
    printf "Max: %.3fms\n", max;
    printf "Avg: %.3fms\n", sum/count;
}' times_without_update.txt
