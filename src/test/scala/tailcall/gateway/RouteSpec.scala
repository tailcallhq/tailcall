package tailcall.gateway

import tailcall.gateway.ast.Route
import tailcall.gateway.ast.Route.Segment.{Literal, Param}
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{Gen, ZIOSpecDefault, assertZIO, checkAll}

object RouteSpec extends ZIOSpecDefault {
  val syntax = Route.syntax.route

  override def spec = suite("route")(test("segments") {
    val input: Seq[(String, List[Route.Segment])] = Seq(
      "/a"              -> (Literal("a") :: Nil),
      "/a/b"            -> (Literal("a") :: Literal("b") :: Nil),
      "/a/b/c"          -> (Literal("a") :: Literal("b") :: Literal("c") :: Nil),
      "/a/b/${c}"       -> (Literal("a") :: Literal("b") :: Param("c") :: Nil),
      "/a/${b}/${c}"    -> (Literal("a") :: Param("b") :: Param("c") :: Nil),
      "/${a}/${b}/${c}" -> (Param("a") :: Param("b") :: Param("c") :: Nil),
      "/${a}/${b}"      -> (Param("a") :: Param("b") :: Nil),
      "/${a}"           -> (Param("a") :: Nil)
    )
    checkAll(Gen.fromIterable(input)) { case (input, expected) =>
      val parsed = ZIO.fromEither(syntax.parseString(input)).map(_.segments)
      assertZIO(parsed)(equalTo(expected))
    }
  })
}
