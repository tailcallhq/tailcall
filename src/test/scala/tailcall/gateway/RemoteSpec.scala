package tailcall.gateway

import tailcall.gateway.remote.Remote
import zio.test.Assertion.{equalTo, isFalse, isTrue}
import zio.test.{ZIOSpecDefault, assertZIO}

object RemoteSpec extends ZIOSpecDefault {
  def spec = suite("Remote")(
    suite("math")(
      test("add") {
        val program = Remote(1) + Remote(2)
        assertZIO(program.eval)(equalTo(3))
      },
      test("subtract") {
        val program = Remote(1) - Remote(2)
        assertZIO(program.eval)(equalTo(-1))
      },
      test("multiply") {
        val program = Remote(2) * Remote(3)
        assertZIO(program.eval)(equalTo(6))
      },
      test("divide") {
        val program = Remote(6) / Remote(3)
        assertZIO(program.eval)(equalTo(2))
      },
      test("modulo") {
        val program = Remote(7) % Remote(3)
        assertZIO(program.eval)(equalTo(1))
      }
    ),
    suite("logical")(
      test("and") {
        val program = Remote(true) && Remote(true)
        assertZIO(program.eval)(isTrue)
      },
      test("or") {
        val program = Remote(true) || Remote(false)
        assertZIO(program.eval)(isTrue)
      },
      test("not") {
        val program = !Remote(true)
        assertZIO(program.eval)(isFalse)
      }
    ),
    suite("equals")(
      test("equal") {
        val program = Remote(1) =:= Remote(1)
        assertZIO(program.eval)(isTrue)
      },
      test("not equal") {
        val program = Remote(1) =:= Remote(2)
        assertZIO(program.eval)(isFalse)
      }
    ),
    suite("diverge")(
      test("isTrue") {
        val program = Remote(true).diverge(Remote("Yes"), Remote("No"))
        assertZIO(program.eval)(equalTo("Yes"))
      },
      test("isFalse") {
        val program = Remote(false).diverge(Remote("Yes"), Remote("No"))
        assertZIO(program.eval)(equalTo("No"))
      }
    ),
    suite("indexSeq")(
      test("concat") {
        val program = Remote(IndexedSeq(1, 2)) ++ Remote(IndexedSeq(3, 4))
        assertZIO(program.eval)(equalTo(IndexedSeq(1, 2, 3, 4)))
      },
      test("reverse") {
        val program = Remote(IndexedSeq(1, 2, 3)).reverse
        assertZIO(program.eval)(equalTo(IndexedSeq(3, 2, 1)))
      },
      test("length") {
        val program = Remote(IndexedSeq(1, 2, 3)).length
        assertZIO(program.eval)(equalTo(3))
      },
      test("indexOf") {
        val program = Remote(IndexedSeq(1, 2, 3)).indexOf(Remote(2))
        assertZIO(program.eval)(equalTo(1))
      }
    )
  )
}
