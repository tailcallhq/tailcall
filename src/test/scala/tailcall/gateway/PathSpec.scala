package tailcall.gateway

import tailcall.gateway.ast.Path
import tailcall.gateway.ast.Path.Segment.{Literal, Param}
import zio.ZIO
import zio.test.Assertion.equalTo
import zio.test.{Gen, ZIOSpecDefault, assertZIO, checkAll}

object PathSpec extends ZIOSpecDefault {
  val syntax = Path.syntax.route

  override def spec =
    suite("path")(test("segments") {
      val input: Seq[(String, List[Path.Segment])] = Seq(
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
