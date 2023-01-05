import tailcall.gateway.adt.Route.Segment.{Literal, Param}
import tailcall.gateway.adt.Route
import zio.test.Assertion.equalTo
import zio.test.{Gen, ZIOSpecDefault, assertZIO, checkAll}
import zio.{Chunk, ZIO}

object RouteSpec extends ZIOSpecDefault {
  val syntax = Route.syntax.route

  override def spec = suite("route")(test("segments") {
    val input = Seq(
      "/a"              -> Chunk(Literal("a")),
      "/a/b"            -> Chunk(Literal("a"), Literal("b")),
      "/a/b/c"          -> Chunk(Literal("a"), Literal("b"), Literal("c")),
      "/a/b/${c}"       -> Chunk(Literal("a"), Literal("b"), Param("c")),
      "/a/${b}/${c}"    -> Chunk(Literal("a"), Param("b"), Param("c")),
      "/${a}/${b}/${c}" -> Chunk(Param("a"), Param("b"), Param("c")),
      "/${a}/${b}"      -> Chunk(Param("a"), Param("b")),
      "/${a}"           -> Chunk(Param("a")),
    )
    checkAll(Gen.fromIterable(input)) { case (input, expected) =>
      val parsed = ZIO.fromEither(syntax.parseString(input)).map(_.segments)
      assertZIO(parsed)(equalTo(expected))
    }
  })
}
