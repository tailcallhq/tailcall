use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use tailcall_hasher::TailcallHasher;
use super::Queries;
use crate::core::config::Config;

#[derive(Clone, Copy, Debug)]
pub struct TypeName<'a> {
    val:  &'a str,
    depth: usize,
}

impl Eq for TypeName<'_> {}

impl PartialEq for TypeName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl Hash for TypeName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

impl<'a> TypeName<'a> {
    pub fn new(name: &'a str, depth: usize) -> Self {
        Self {
            val: name,
            depth,
        }
    }
    pub fn as_str(self) -> &'a str {
        self.val
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
}
impl Display for TypeName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FieldName<'a>(&'a str);
impl<'a> FieldName<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }
    pub fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Depth(usize);

impl Depth {
    fn new() -> Self {
        Self(0)
    }
    fn get(&self) -> usize {
        self.0
    }
}

impl Iterator for Depth {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let Depth(ref mut depth) = *self;
        *depth += 1;
        Some(*depth)
    }
}

pub struct Identifier<'a> {
    config: &'a Config,
    visited: HashSet<u64>,
}

impl<'a> Identifier<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config, visited: Default::default() }
    }

    pub fn identify(mut self) -> Queries<'a> {
        if let Some(query) = &self.config.schema.query {
            self.find_fan_out(query, false, 0)
        } else {
            Default::default()
        }
    }
    #[inline(always)]
    fn find_fan_out(&mut self, type_name: &'a str, is_list: bool, depth: usize) -> Queries<'a> {
        let config = self.config;
        let type_name: TypeName = TypeName::new(type_name, depth);
        let mut ans: HashMap<TypeName, HashSet<(FieldName, TypeName)>> = HashMap::new();

        if let Some(type_) = config.find_type(type_name.as_str()) {
            for (field_name, field) in type_.fields.iter() {
                let cur: FieldName = FieldName(field_name.as_str());
                let ty_of = TypeName::new(field.type_of.as_str(), 0);
                let tuple: (FieldName, TypeName) = (cur, ty_of);
                let field_conditions = field.has_resolver() && !field.has_batched_resolver() && is_list;

                let mut hasher = TailcallHasher::default();
                type_name.hash(&mut hasher);
                cur.as_str().hash(&mut hasher);
                ty_of.as_str().hash(&mut hasher);
                field_conditions.hash(&mut hasher);

                let hash = hasher.finish();

                let condition = self.visited.contains(&hash);

                if condition {
                    continue;
                } else {
                    self.visited.insert(hash);
                }

                if field_conditions {
                    ans.entry(type_name).or_default().insert(tuple);
                } else {
                    let next = self.find_fan_out(&field.type_of, field.list || is_list, depth + 1);
                    for (k, v) in next.map() {
                        ans.entry(*k).or_default().extend(v);
                        ans.entry(type_name).or_default().insert(tuple);
                    }
                }
            }
        }

        Queries::new(ans, type_name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::core::config::npo::Queries;
    use crate::core::config::{Config, Field, Http, Type};
    use crate::core::valid::Validator;

    macro_rules! assert_eq_map {
        ($actual:expr, $expected_vec:expr) => {{
            let mut expected: HashMap<TypeName, HashSet<(FieldName, TypeName)>> = HashMap::new();
            for vec in $expected_vec {
                for value in vec {
                    let (key, (value, ty_of)) = value;
                    let key = TypeName::new(key, 0);
                    let value = (FieldName(value), TypeName::new(ty_of, 0));
                    expected
                        .entry(key)
                        .or_default()
                        .insert(value);
                }
            }

            assert_eq!($actual, Queries::new(expected, ($actual).root()));
        }};
    }

    #[test]
    fn test_nplusone_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default()
                        .type_of("F2".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);
        let actual = config.n_plus_one();
        let expected = vec![vec![("Query", ("f1", "F1")), ("F1", ("f2", "F2"))]];

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_batched_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default()
                        .type_of("F2".to_string())
                        .into_list()
                        .http(Http { batch_key: vec!["id".into()], ..Default::default() }),
                )]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<_>> = vec![];
        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_nested_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default().type_of("F2".to_string()).into_list(),
                )]),
            ),
            (
                "F2",
                Type::default().fields(vec![(
                    "f3",
                    Field::default().type_of("F3".to_string()).into_list(),
                )]),
            ),
            (
                "F3",
                Type::default().fields(vec![(
                    "f4",
                    Field::default()
                        .type_of("String".to_string())
                        .http(Http::default()),
                )]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![vec![
            ("Query", ("f1", "F1")),
            ("F1", ("f2", "F2")),
            ("F2", ("f3", "F3")),
            ("F3", ("f4", "String")),
        ]];

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_nested_resolvers_non_list_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default().type_of("F2".to_string()).into_list(),
                )]),
            ),
            (
                "F2",
                Type::default().fields(vec![(
                    "f3",
                    Field::default().type_of("F3".to_string()).into_list(),
                )]),
            ),
            (
                "F3",
                Type::default().fields(vec![(
                    "f4",
                    Field::default()
                        .type_of("String".to_string())
                        .http(Http::default()),
                )]),
            ),
        ]);

        let expected = vec![vec![
            ("Query", ("f1", "F1")),
            ("F1", ("f2", "F2")),
            ("F2", ("f3", "F3")),
            ("F3", ("f4", "String")),
        ]];
        let actual = config.n_plus_one();

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_nested_resolvers_without_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default().type_of("F2".to_string()).into_list(),
                )]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let expected: Vec<Vec<_>> = vec![];
        let actual = config.n_plus_one();

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_cycles() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![
                    ("f1", Field::default().type_of("F1".to_string())),
                    ("f2", Field::default().type_of("F2".to_string()).into_list()),
                ]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<_>> = vec![];

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_cycles_with_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![
                    ("f1", Field::default().type_of("F1".to_string()).into_list()),
                    (
                        "f2",
                        Field::default()
                            .type_of("String".to_string())
                            .http(Http::default()),
                    ),
                ]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![
            vec![
                ("Query", ("f1", "F1")),
                ("F1", ("f1", "F1")),
                ("F1", ("f2", "String")),
            ],
            vec![("Query", ("f1", "F1")), ("F1", ("f2", "String"))],
        ];

        assert_eq_map!(actual, expected);
    }
    #[test]
    fn test_nplusone_nested_non_list() {
        let f_field = Field::default()
            .type_of("F".to_string())
            .http(Http::default());

        let config = Config::default().query("Query").types(vec![
            ("Query", Type::default().fields(vec![("f", f_field)])),
            (
                "F",
                Type::default().fields(vec![(
                    "g",
                    Field::default()
                        .type_of("G".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "G",
                Type::default().fields(vec![("e", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = Vec::<Vec<_>>::new();

        assert_eq_map!(actual, expected);
    }
    #[test]
    fn test_jp_config() {
        let config = r#"
        schema
  @server(port: 8000, headers: {cors: {allowOrigins: ["*"], allowHeaders: ["*"], allowMethods: [POST, GET, OPTIONS]}})
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
  query: Query
  mutation: Mutation
}

type Query {
  posts: [Post] @http(path: "/posts")
  users: [User] @http(path: "/users")
  user(id: ID!): User @http(path: "/users/{{.args.id}}")
  post(id: ID!): Post @http(path: "/posts/{{.args.id}}")
  getUserIdOrEmail(id: ID!): UserIdOrEmail @http(path: "/users/{{.args.id}}")
}

type UserId {
  id: ID!
}

type UserEmail {
  email: String!
}

union UserIdOrEmail = UserId | UserEmail

type Mutation {
  createUser(user: InputUser!): User @http(method: POST, path: "/users", body: "{{args.user}}")
  createPost(post: InputPost!): Post @http(method: POST, path: "/posts", body: "{{args.post}}")
}

input InputPost {
  id: ID = 101
  userId: ID!
  title: String!
  body: String!
}

input InputUser {
  id: ID!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type User {
  id: ID!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  address: Address
  todo: Todo @http(path: "/todos/{{.value.id}}")
}

type Todo {
  id: ID!
  userId: ID!
  title: String
  completed: Boolean
}

type Address {
  street: String
  suite: String
  city: String
  zipcode: String
}

type Post {
  id: ID!
  userId: ID!
  title: String!
  body: String!
  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])
}

        "#;
        let config = Config::from_sdl(config).to_result().unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        println!("{}", formatted);
    }
}

// query { posts { user } }
// query { users { todo } }