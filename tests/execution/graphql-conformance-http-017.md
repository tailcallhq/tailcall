# Complex fragments.

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  edibleAnimals: [EdibleAnimals] @http(url: "http://upstream/edible-animals")
  allAnimals: [Animal] @http(url: "http://upstream/all-animals")
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
  canProduceMilk: Boolean!
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
  isForBacon: Boolean!
}

type Boar implements Animal & WildAnimal {
  id: ID!
  legs: Int!
  sound: String!
  dangerous: Boolean!
  blackBoar: Boolean!
}

type Deer implements Animal & WildAnimal {
  id: ID!
  legs: Int!
  sound: String!
  dangerous: Boolean!
  hasAntlers: Boolean!
}

type Dog implements Animal & DomesticAnimal & Pet {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
  owner: String!
  size: Int!
}

type Cat implements Animal & DomesticAnimal & Pet {
  id: ID!
  legs: Int!
  sound: String!
  weight: Int!
  owner: String!
  hasFur: Boolean!
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
      - Cat:
          id: cat-1
          legs: 4
          sound: meow
          weight: 2
          owner: John
          hasFur: true
      - Dog:
          id: dog-2
          legs: 4
          sound: woof
          weight: 2
          owner: Steve
          size: 12
      - Salmon:
          id: salmon-1
          legs: 0
          sound: ...
          length: 2
      - Salmon:
          id: salmon-2
          legs: 0
          sound: ...
          length: 1
      - Pig:
          id: pig-1
          legs: 4
          sound: oik
          weight: 24
          isForBacon: false
      - Pig:
          id: pig-2
          legs: 4
          sound: oik
          weight: 41
          isForBacon: true
- request:
    method: GET
    url: http://upstream/edible-animals
  expectedHits: 1
  response:
    status: 200
    body:
      - Salmon:
          id: salmon-1
          legs: 0
          sound: ...
          length: 2
      - Salmon:
          id: salmon-2
          legs: 0
          sound: ...
          length: 1
      - Pig:
          id: pig-1
          legs: 4
          sound: oik
          weight: 24
      - Pig:
          id: pig-2
          legs: 4
          sound: oik
          weight: 41
          isForBacon: false
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
