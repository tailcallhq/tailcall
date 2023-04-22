package tailcall.runtime

import tailcall.runtime.internal.TValid
import zio.Scope
import zio.test.{Spec, TestEnvironment, ZIOSpecDefault, assertTrue}

object TValidSpec extends ZIOSpecDefault {
  override def spec: Spec[TestEnvironment with Scope, Any] =
    suite("TValid")(
      suite("foreach")(
        test("combine errors") {
          val program = TValid.foreach(List(1, 2, 3))(i => TValid.fail(i + 1))
          assertTrue(program == TValid.fail(2, 3, 4))
        },
        test("fail if any error is found") {
          val list    = List(TValid.succeed(1), TValid.fail(-1), TValid.succeed(1))
          val program = TValid.foreach(list)(identity(_))
          assertTrue(program == TValid.fail(-1))
        },
      ),
      suite("fold")(test("fail fast") {
        val program = TValid.fold(List(1, 2, 3), 0)((b, a) => TValid.fail(a + b))
        assertTrue(program == TValid.fail(1))
      }),
      test("zipPar") {
        val program = TValid.fail(1).zipPar(TValid.fail(2))((_, _) => 1)
        assertTrue(program == TValid.fail(1, 2))
      },
    )
}
