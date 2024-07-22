# Http with args as body

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {maxSize: 1000, delay: 10}) {
  query: Query
}

type Query {
  transactions: [Transaction!]! @http(path: "/transactions")
}

type Transaction {
  id: ID!
  slug: String!
  bank_account_id: String!
  BankAccounts: [BankAccount!]!
    @http(
      path: "/v1/bank_accounts"
      query: [{key: "bank_account_ids[]", value: "{{.value.bank_account_id}}"}]
      batchKey: [{query: "bank_account_ids[]", object: "id"}]
    )
}

type BankAccount {
  id: ID!
  bic: String!
  iban: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/transactions
  response:
    status: 200
    body:
      [
        {
          "id": "7964963e-06e8-46a7-b228-04fc08fd020f",
          "slug": "slug-tx-1",
          "bank_account_id": "441ff0f9-c00d-45cc-8bbf-5a8fffa01798",
        },
        {
          "id": "140b5e4d-b9c0-40f7-97ec-313c9bda2b00",
          "slug": "slug-tx-2",
          "bank_account_id": "ea2efff5-6d7c-4cf3-bada-8447e139a864",
        },
      ]
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/v1/bank_accounts?bank_account_ids[]=441ff0f9-c00d-45cc-8bbf-5a8fffa01798&bank_account_ids%5B%5D=ea2efff5-6d7c-4cf3-bada-8447e139a864
  response:
    status: 200
    body:
      [
        {"id": "441ff0f9-c00d-45cc-8bbf-5a8fffa01798", "iban": "iban1", "bic": "bic1"},
        {"id": "441ff0f9-c00d-45cc-8bbf-5a8fffa01798", "iban": "iban2", "bic": "bic3"},
        {"id": "ea2efff5-6d7c-4cf3-bada-8447e139a864", "iban": "iban3", "bic": "bic3"},
      ]
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |-
      {
        transactions {
          id,
          bank_account_id,
          BankAccounts {
            bic
            iban
            id
          }
        }
      }
```
