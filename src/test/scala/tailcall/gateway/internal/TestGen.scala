package tailcall.gateway.internal

import tailcall.gateway.adt.Config
import tailcall.gateway.adt.Config._
import zio.test.Gen

object TestGen {
  def genName: Gen[Any, String] = Gen.alphaNumericStringBounded(3, 5)

  def genBaseURL: Gen[Any, String] = genName

  def genVersion: Gen[Any, String] = genName

  def genScalar: Gen[Any, Schema] = Gen.fromIterable(List(Schema.Str, Schema.Int, Schema.Null))

  def genField: Gen[Any, Schema.Field] =
    for {
      name     <- genName
      kind     <- genScalar
      required <- Gen.boolean
    } yield Schema.Field(name, kind, required)

  def genObj: Gen[Any, Schema] = Gen.listOfBounded(2, 5)(genField).map(fields => Schema.Obj(fields))

  def genSchema: Gen[Any, Config.Schema] = genObj

  def genServer: Gen[Any, Config.Server] = genBaseURL.map(baseURL => Config.Server(baseURL))

  def genMethod: Gen[Any, Method] = Gen.oneOf(
    Gen.const(Method.GET),
    Gen.const(Method.POST),
    Gen.const(Method.PUT),
    Gen.const(Method.DELETE),
  )

  def genSegment: Gen[Any, Route.Segment] = Gen
    .oneOf(genName.map(Route.Segment.Literal), genName.map(Route.Segment.Param))

  def genRoute: Gen[Any, Route] = Gen.listOf(genSegment).map(Route(_))

  def genHttp: Gen[Any, Http] =
    for {
      path   <- genRoute
      method <- genMethod
    } yield Http(path, method)

  def genEndpoints: Gen[Any, Config.Endpoint] =
    for {
      http   <- genHttp
      input  <- Gen.option(genSchema)
      output <- genSchema
    } yield Endpoint(http, input, output)

  def genGraphQL: Gen[Any, Config.GraphQL] = Gen.const(Config.GraphQL())

  def genConfig: Gen[Any, Config] =
    for {
      version   <- genVersion
      server    <- genServer
      endpoints <- Gen.listOfN(2)(genEndpoints)
      graphQL   <- genGraphQL
    } yield Config(version, server, endpoints, graphQL)
}
