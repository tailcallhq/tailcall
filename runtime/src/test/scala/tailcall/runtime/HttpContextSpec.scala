package tailcall.runtime

import tailcall.TailcallSpec
import tailcall.runtime.service.HttpContext
import zio.durationInt
import zio.test.Assertion._
import zio.test._
object HttpContextSpec extends TailcallSpec {
  override def spec =
    suite("HttpContext State")(
      test("state set") {
        val state = HttpContext.update(_ => HttpContext.State(Some(1 second)))
        assertZIO(state)(equalTo(HttpContext.State(Some(1 second))))
      },
      test("state get") {
        val program = for {
          _ <- HttpContext.update(_ => HttpContext.State(Some(2 second)))
          s <- HttpContext.getState
        } yield s
        assertZIO(program)(equalTo(HttpContext.State(Some(2 second))))
      },
      test("state get withCacheMaxAge > previous") {
        val program = for {
          state <- HttpContext.update(_ => HttpContext.State(Some(2 second)))
          _     <- HttpContext.update(_ => state.withCacheMaxAge(3 second))
          s     <- HttpContext.getState
        } yield s
        assertZIO(program)(equalTo(HttpContext.State(Some(2 second))))
      },
      test("state get withCacheMaxAge < previous") {
        val program = for {
          state <- HttpContext.update(_ => HttpContext.State(Some(2 second)))
          _     <- HttpContext.update(_ => state.withCacheMaxAge(1 second))
          s     <- HttpContext.getState
        } yield s
        assertZIO(program)(equalTo(HttpContext.State(Some(1 second))))
      },
    ).provideLayer(HttpContext.default)
}
