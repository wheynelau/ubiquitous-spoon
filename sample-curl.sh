curl -X POST http://localhost:3000/shorten \
-H "Content-Type: application/json" \
-d '{"url":"https://www.google.com","expiration_date":"2023-12-31"}'