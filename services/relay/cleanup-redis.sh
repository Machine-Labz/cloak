#!/bin/bash
# Clean up stale Redis queue entries

echo "🧹 Cleaning up Redis queues..."

# Connect to Redis container and flush the relay queues
docker exec -it cloak-redis redis-cli << EOF
DEL cloak:relay:jobs
DEL cloak:relay:retry
DEL cloak:relay:processing
DEL cloak:relay:dlq
SAVE
EOF

echo "✅ Redis queues cleaned!"
echo ""
echo "Queue status:"
docker exec -it cloak-redis redis-cli << EOF
ZCARD cloak:relay:jobs
ZCARD cloak:relay:retry
LLEN cloak:relay:processing
LLEN cloak:relay:dlq
EOF

