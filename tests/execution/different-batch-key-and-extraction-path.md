# different batch key and extraction path

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {maxSize: 1000, delay: 1}) {
  query: Query
}

type Query {
  foo: Foo @http(path: "/transactions")
}

type Foo {
  transactions: [Transaction]
}

type Transaction {
  id: ID
  slug: String
  bank_account_id: String
  BankAccounts: [BankAccount]
    @http(
      path: "/v1/bank_accounts"
      query: [{key: "bank_account_ids[]", value: "{{.value.bank_account_id}}"}]
      batch: {
        batchKey: "bank_account_ids[]",
        extractionPath: ["bank_accounts","id"]
      }
    )
}

type BankAccount {
  id: ID
  bic: String
  iban: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/transactions
  response:
    status: 200
    body:
      transactions:
        - id: "7964963e-06e8-46a7-b228-04fc08fd020f"
          slug: "slug-tx-1"
          bank_account_id: "441ff0f9-c00d-45cc-8bbf-5a8fffa01798"
        - id: "140b5e4d-b9c0-40f7-97ec-313c9bda2b00"
          slug: "slug-tx-2"
          bank_account_id: "ea2efff5-6d7c-4cf3-bada-8447e139a864"
      meta:
        current_page: 1
        next_page: 2
        prev_page: null
        total_pages: 2
        total_count: 4
        per_page: 2

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/v1/bank_accounts?bank_account_ids[]=441ff0f9-c00d-45cc-8bbf-5a8fffa01798&bank_account_ids%5B%5D=ea2efff5-6d7c-4cf3-bada-8447e139a864
  response:
    status: 200
    body:
      bank_accounts:
        - id: "441ff0f9-c00d-45cc-8bbf-5a8fffa01798"
          iban: "iban1"
          bic: "bic1"
        - id: "441ff0f9-c00d-45cc-8bbf-5a8fffa01798"
          iban: "iban2"
          bic: "bic3"
        - id: "ea2efff5-6d7c-4cf3-bada-8447e139a864"
          iban: "iban3"
          bic: "bic3"
      meta:
        current_page: 1
        next_page: 2
        prev_page: null
        total_pages: 2
        total_count: 4
        per_page: 2

```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |-
      {
        foo {
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
      }
```