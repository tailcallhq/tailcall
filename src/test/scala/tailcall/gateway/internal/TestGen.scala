package tailcall.gateway.internal

import tailcall.gateway.ast._
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config._
import zio.test.Gen

object TestGen {
  def genName: Gen[Any, String] = Gen.alphaNumericStringBounded(3, 5)

  def genBaseURL: Gen[Any, String] = genName

  def genVersion: Gen[Any, String] = genName

  def genScalar: Gen[Any, TSchema] = Gen
    .fromIterable(List(TSchema.str, TSchema.int, TSchema.`null`))

  def genField: Gen[Any, TSchema.Field] = for {
    name <- genName
    kind <- genScalar
  } yield TSchema.Field(name, kind)

  def genObj: Gen[Any, TSchema] = Gen.listOfBounded(2, 5)(genField)
    .map(fields => TSchema.Obj(TSchema.Id.Structural, fields))

  def genSchema: Gen[Any, TSchema] = genObj

  def genServer: Gen[Any, Config.Server] = genBaseURL.map(baseURL => Config.Server(baseURL))

  def genMethod: Gen[Any, Method] = Gen.oneOf(
    Gen.const(Method.GET),
    Gen.const(Method.POST),
    Gen.const(Method.PUT),
    Gen.const(Method.DELETE)
  )

  def genPlaceholder: Gen[Any, Placeholder] = for {
    name <- Gen.chunkOf(genName)
  } yield Placeholder(name)

  def genSegment: Gen[Any, Path.Segment] = Gen
    .oneOf(genName.map(Path.Segment.Literal), genPlaceholder.map(Path.Segment.Param(_)))

  def genRoute: Gen[Any, Path] = Gen.listOf(genSegment).map(Path(_))

  def genHttp: Gen[Any, Operation.Http] = for {
    path   <- genRoute
    method <- genMethod
  } yield Operation.Http(path, method)

  def genEndpoints: Gen[Any, Config.ConfigEndpoint] = for {
    http   <- genHttp
    input  <- Gen.option(genSchema)
    output <- genSchema
  } yield ConfigEndpoint(http, input, output)

  def genConnection: Gen[Any, (String, Connection)] = for {
    name     <- genName
    endpoint <- Gen.listOf(genEndpoints)
  } yield (name, Connection(endpoint))

  def genGraphQL: Gen[Any, Config.Specification] = for {
    connections <- Gen.listOf1(genConnection)
  } yield Config.Specification(Map("Query" -> Map.from(connections)))

  def genConfig: Gen[Any, Config] = for {
    version <- genVersion
    server  <- genServer
    graphQL <- genGraphQL
  } yield Config(version, server, graphQL)
}
