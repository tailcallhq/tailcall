package tailcall.gateway

import tailcall.gateway.internal.TValid
import zio.Chunk
import zio.test.Assertion._
import zio.test.{ZIOSpecDefault, _}

object TValidSpec extends ZIOSpecDefault:

  def spec =
    suite("TValid")(
      test("flatMap") {
        val valid = TValid.success(1).flatMap(i => TValid.success(i + 1))
        assert(valid)(equalTo(TValid.success(2)))
      },
      test("flatMap with error") {
        val valid = TValid.success(1).flatMap(_ => TValid.fail("error"))
        assert(valid.errors)(equalTo(Chunk.single("error")))
      },
      test("flatMap ++ with error") {
        val valid = (TValid.success(1) ++ TValid.fail("error")).flatMap(i => TValid.success(i + 1))

        assert(valid.errors)(equalTo(Chunk.single("error"))) &&
        assert(valid.values)(equalTo(Chunk.single(2)))
      },
      test("map") {
        val valid = TValid.success(1).map(_ + 1)
        assert(valid)(equalTo(TValid.success(2)))
      },
      test("map with error") {
        val valid = TValid.fail("error").map(_ => 1)
        assert(valid.errors)(equalTo(Chunk.single("error")))
      },
      test("++") {
        val valid1 = TValid.success(1)
        val valid2 = TValid.success(2)
        val valid3 = valid1 ++ valid2
        assert(valid3)(equalTo(TValid.success(1) ++ TValid.success(2)))
      },
      test("++ with errors") {
        val valid1 = TValid.fail("error1")
        val valid2 = TValid.fail("error2")
        val valid3 = valid1 ++ valid2
        assert(valid3.errors)(equalTo(Chunk.fromIterable(List("error1", "error2"))))
      },
      test("success")(assert(TValid.success(1))(equalTo(TValid(Chunk.empty, Chunk.single(1))))),
      test("fail")(assert(TValid.fail(1))(equalTo(TValid(Chunk.single(1), Chunk.empty)))),
      test("empty")(assert(TValid.empty)(equalTo(TValid(Chunk.empty, Chunk.empty)))),
      test("unit")(assert(TValid.unit)(equalTo(TValid.success(())))),
      test("from") {
        val valid1 = TValid.success(1)
        val valid2 = TValid.success(2)
        val valid3 = TValid.from(List(valid1, valid2))
        assert(valid3)(equalTo(TValid.success(1) ++ TValid.success(2)))
      },
      test("from with errors") {
        val valid1 = TValid.success(1)
        val valid2 = TValid.fail("error")
        val valid3 = TValid.from(List(valid1, valid2))
        assert(valid3.errors)(equalTo(Chunk.single("error")))
      }
    )
