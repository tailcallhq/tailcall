---
skip: true
---

# Complex fragments.

TODO: Skipped because there is a pending case to improve the discriminator.

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  edibleAnimals: EdibleAnimals @http(path: "/edible-animals")
  allAnimals: Animal @http(path: "/all-animals")
}

interface Animal {
  id: ID!
  legs: Int!
  sound: String!
}

interface Bird {
  eggSize: Int!
}

interface Fish {
  length: Int!
}

interface DomesticAnimal {
  weight: Int!
}

interface Pet {
  owner: String!
}

interface WildAnimal {
  dangerous: Boolean!
}

union HuntedAnimals = Boar | Salmon

union FarmingAnimals = Pig | Chicken

union EdibleAnimals = HuntedAnimals | FarmingAnimals

type Cow implements Animal & DomesticAnimal {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
}

type Chicken implements Animal & Bird {
  id: ID!
  legs: Int!
  sound: String!
  eggSize: Int!
}

type Salmon implements Animal & Fish {
  id: ID!
  legs: Int!
  sound: String!
  length: Int!
}

type Pig implements Animal & DomesticAnimal {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
}

type Boar implements Animal & WildAnimal {
  id: ID!
  legs: Int!
  sound: String!
  dangerous: Boolean!
}

type Deer implements Animal & WildAnimal {
  id: ID!
  legs: Int!
  sound: String!
  dangerous: Boolean!
}

type Dog implements Animal & DomesticAnimal & Pet {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
  owner: String!
}

type Cat implements Animal & DomesticAnimal & Pet {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
  owner: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/all-animals
  expectedHits: 1
  response:
    status: 200
    body:
      - id: cat-1
        legs: 4
        sound: meow
        weight: 2
        owner: John
      - id: dog-2
        legs: 4
        sound: woof
        weight: 2
        owner: Steve
      - id: salmon-1
        legs: 0
        sound: ...
        length: 2
      - id: salmon-2
        legs: 0
        sound: ...
        length: 1
      - id: pig-1
        legs: 4
        sound: oik
        weight: 24
      - id: pig-2
        legs: 4
        sound: oik
        weight: 41
- request:
    method: GET
    url: http://upstream/edible-animals
  expectedHits: 1
  response:
    status: 200
    body:
      - id: salmon-1
        legs: 0
        sound: ...
        length: 2
      - id: salmon-2
        legs: 0
        sound: ...
        length: 1
      - id: pig-1
        legs: 4
        sound: oik
        weight: 24
      - id: pig-2
        legs: 4
        sound: oik
        weight: 41
```

```yml @test
# Positve
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        allAnimals {
          ...animalsFragment
        }
      }

      fragment animalsFragment on Animal {
        id
        sound
        ...domesticFragment
        ...petFragment
        ... on Cat {
          legs
        }
      }

      fragment domesticFragment on DomesticAnimal {
        weight
      }

      fragment petFragment on Pet {
        owner
      }
# Positive
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        edibleAnimals {
          ...edibleFragment
        }
      }

      fragment edibleFragment on EdibleAnimals {
        ... on Animal {
          id
        }
        ...domesticFragment
        ...boarFragment
      }

      fragment boarFragment on Boar {
        sound
        dangerous
      }
```
