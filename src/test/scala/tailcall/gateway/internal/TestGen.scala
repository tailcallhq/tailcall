package tailcall.gateway.internal

import tailcall.gateway.ast._
import tailcall.gateway.dsl.json.Config
import tailcall.gateway.dsl.json.Config._
import tailcall.gateway.http.Method
import zio.test.Gen

object TestGen {
  def genName: Gen[Any, String] = fromIterableRandom("body", "completed", "email", "id", "name", "title", "url")

  def genBaseURL: Gen[Any, String] = genName

  def genVersion: Gen[Any, Int] = Gen.int(0, 10)

  def genScalar: Gen[Any, TSchema] = Gen.fromIterable(List(TSchema.str, TSchema.int, TSchema.`null`))

  def genField: Gen[Any, TSchema.Field] =
    for {
      name <- genName
      kind <- genScalar
    } yield TSchema.Field(name, kind)

  def genObj: Gen[Any, TSchema] = Gen.listOfN(2)(genField).map(fields => TSchema.obj(fields))

  def genSchema: Gen[Any, TSchema] = genObj

  def genServer: Gen[Any, Config.Server] = genBaseURL.map(baseURL => Config.Server(baseURL))

  def genMethod: Gen[Any, Method] =
    Gen.oneOf(Gen.const(Method.GET), Gen.const(Method.POST), Gen.const(Method.PUT), Gen.const(Method.DELETE))

  def genMustache: Gen[Any, Mustache] =
    for { name <- Gen.chunkOfN(2)(genName) } yield Mustache(name: _*)

  def genSegment: Gen[Any, Path.Segment] =
    Gen.oneOf(genName.map(Path.Segment.Literal), genMustache.map(Path.Segment.Param(_)))

  def genPath: Gen[Any, Path] = Gen.listOfN(2)(genSegment).map(Path(_))

  def genHttp: Gen[Any, Step.Http] =
    for {
      path   <- genPath
      method <- genMethod
      input  <- Gen.option(genSchema)
      output <- Gen.option(genSchema)
    } yield Step.Http(path, method, input, output)

  def genStep: Gen[Any, Config.Step] =
    for { http <- genHttp } yield http

  def genFieldDefinition: Gen[Any, Field] =
    for {
      typeName <- genTypeName
      steps    <- Gen.option(Gen.listOf(genStep))
    } yield Field(as = typeName, steps = steps)

  def fromIterableRandom[A](seq: A*): Gen[Any, A] =
    Gen.fromRandom { random =>
      val list = seq.toVector
      random.nextIntBetween(0, list.length - 1).map(list(_))
    }

  def genTypeName: Gen[Any, String] = {
    fromIterableRandom("Query", "User", "Post", "Comment", "Album", "Photo", "Todo")
  }

  def schemaDefinition: Gen[Any, SchemaDefinition] = Gen.const(SchemaDefinition(Option("Query"), None))

  def genGraphQL: Gen[Any, Config.GraphQL] =
    for {
      map    <- Gen.mapOfN(2)(genTypeName, Gen.mapOfN(2)(genName, genFieldDefinition))
      schema <- schemaDefinition
    } yield Config.GraphQL(schema, map)

  def genConfig: Gen[Any, Config] =
    for {
      version <- genVersion
      server  <- genServer
      graphQL <- genGraphQL
    } yield Config(version, server, graphQL)
}
