set +x

if [ -z  "$TEST_TARGET" ]; then
    echo "specify test target TEST_TARGET"
    exit 1
fi

echo "# Test for ${TEST_TARGET}" > ${TEST_TARGET}.md

for n_concur in 100 500 1000 2500 5000 7500 10000
do
    for n_reqs in 100 500 1000 2500
    do
        printf "## Number of requests \n### ${n_reqs}x \n### concurrency: ${n_concur}\n"  | tee -a ${TEST_TARGET}.md
        go test --bench=. -benchtime=${n_reqs}x -parallel ${n_concur} | tee -a ${TEST_TARGET}.md
        sleep 2
        # just to be nice
    done
done
