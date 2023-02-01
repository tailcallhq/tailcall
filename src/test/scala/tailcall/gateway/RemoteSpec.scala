package tailcall.gateway

import tailcall.gateway.internal.RemoteAssertion
import tailcall.gateway.remote.Remote
import zio.test.Assertion.{equalTo, isFalse, isTrue}
import zio.test.ZIOSpecDefault

object RemoteSpec extends ZIOSpecDefault with RemoteAssertion {

  def spec = suite("Remote")(
    suite("math")(
      test("add") {
        val program = Remote(1) + Remote(2)
        assertRemote(program)(equalTo(3))
      },
      test("subtract") {
        val program = Remote(1) - Remote(2)
        assertRemote(program)(equalTo(-1))
      },
      test("multiply") {
        val program = Remote(2) * Remote(3)
        assertRemote(program)(equalTo(6))
      },
      test("divide") {
        val program = Remote(6) / Remote(3)
        assertRemote(program)(equalTo(2))
      },
      test("modulo") {
        val program = Remote(7) % Remote(3)
        assertRemote(program)(equalTo(1))
      }
    ),
    suite("logical")(
      test("and") {
        val program = Remote(true) && Remote(true)
        assertRemote(program)(isTrue)
      },
      test("or") {
        val program = Remote(true) || Remote(false)
        assertRemote(program)(isTrue)
      },
      test("not") {
        val program = !Remote(true)
        assertRemote(program)(isFalse)
      }
    ),
    suite("equals")(
      test("equal") {
        val program = Remote(1) =:= Remote(1)
        assertRemote(program)(isTrue)
      },
      test("not equal") {
        val program = Remote(1) =:= Remote(2)
        assertRemote(program)(isFalse)
      }
    ),
    suite("diverge")(
      test("isTrue") {
        val program = Remote(true).diverge(Remote("Yes"), Remote("No"))
        assertRemote(program)(equalTo("Yes"))
      },
      test("isFalse") {
        val program = Remote(false).diverge(Remote("Yes"), Remote("No"))
        assertRemote(program)(equalTo("No"))
      }
    ),
    suite("indexSeq")(
      test("concat") {
        val program = Remote(IndexedSeq(1, 2)) ++ Remote(IndexedSeq(3, 4))
        assertRemote(program)(equalTo(IndexedSeq(1, 2, 3, 4)))
      },
      test("reverse") {
        val program = Remote(IndexedSeq(1, 2, 3)).reverse
        assertRemote(program)(equalTo(IndexedSeq(3, 2, 1)))
      },
      test("length") {
        val program = Remote(IndexedSeq(1, 2, 3)).length
        assertRemote(program)(equalTo(3))
      },
      test("indexOf") {
        val program = Remote(IndexedSeq(1, 2, 3)).indexOf(Remote(2))
        assertRemote(program)(equalTo(1))
      },
      test("function") {
        val function = Remote.fromFunction[Int, Int](_.increment)
        val program  = function(Remote(1))
        assertRemote(program)(equalTo(2))
      },
      test("filter") {
        val program = Remote(IndexedSeq(1, 2, 3, 4)).filter[Int](r => r % Remote(2) =:= Remote(0))
        assertRemote(program)(equalTo(IndexedSeq(2, 4)))
      }
    )
  )
}
