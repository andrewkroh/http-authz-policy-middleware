#!/bin/bash
set -e

echo "=== Traefik WASM Authorization Plugin Integration Test ==="
echo ""

# Wait for Traefik to be ready
echo "Waiting for Traefik to start..."
for i in {1..30}; do
    if curl -s http://localhost:8080/ping > /dev/null 2>&1; then
        echo "✅ Traefik is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Traefik did not start in time"
        exit 1
    fi
    sleep 1
done

echo ""
echo "--- Test 1: Authorized request with platform-eng team (expect 200) ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "X-Auth-User-Teams: platform-eng,devops" \
    http://localhost:8080/allowed)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Test 1 passed: Got HTTP $HTTP_CODE"
else
    echo "❌ Test 1 failed: Expected HTTP 200, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "--- Test 2: Unauthorized request with marketing team (expect 403) ---"
RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
    -H "X-Auth-User-Teams: marketing" \
    http://localhost:8080/denied)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d')

if [ "$HTTP_CODE" = "403" ]; then
    echo "✅ Test 2 passed: Got HTTP $HTTP_CODE"
else
    echo "❌ Test 2 failed: Expected HTTP 403, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "--- Test 3: Verify deny body message ---"
if echo "$BODY" | grep -q "requires platform-eng team membership"; then
    echo "✅ Test 3 passed: Deny body contains expected message"
else
    echo "❌ Test 3 failed: Expected deny body message not found"
    echo "Got: $BODY"
    exit 1
fi

echo ""
echo "--- Test 4: Request without team header (expect 403) ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    http://localhost:8080/denied)

if [ "$HTTP_CODE" = "403" ]; then
    echo "✅ Test 4 passed: Got HTTP $HTTP_CODE"
else
    echo "❌ Test 4 failed: Expected HTTP 403, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "--- Test 5: GET method allowed (expect 200) ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X GET \
    http://localhost:8080/method-test)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Test 5 passed: Got HTTP $HTTP_CODE"
else
    echo "❌ Test 5 failed: Expected HTTP 200, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "--- Test 6: POST method denied (expect 405) ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST \
    http://localhost:8080/method-test)

if [ "$HTTP_CODE" = "405" ]; then
    echo "✅ Test 6 passed: Got HTTP $HTTP_CODE"
else
    echo "❌ Test 6 failed: Expected HTTP 405, got $HTTP_CODE"
    exit 1
fi

echo ""
echo "=== All integration tests passed! ==="
