-- Aevora parallel local-worker example.
-- Run with: --script scripts/examples/worker-info.luau --instances 4

local sum = 0
for value = 1, 100 do
    sum += value
end

print(string.format(
    "%s finished its local test (worker %d of %d, sum=%d)",
    Aevora.instanceName,
    Aevora.instanceIndex,
    Aevora.instanceCount,
    sum
))
