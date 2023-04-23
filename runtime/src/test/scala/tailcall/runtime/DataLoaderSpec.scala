package tailcall.runtime

import tailcall.runtime.service.DataLoader
import zio._
import zio.test.TestAspect.timeout
import zio.test._

object DataLoaderSpec extends ZIOSpecDefault {
  private val failsFirstDL = for {
    ref <- Ref.make(true)
    dl  <- DataLoader.make[RuntimeFlags] { _ =>
      for {
        fail <- ref.get
        _    <- ref.set(false)
        _    <- ZIO.fail("Failure").when(fail)

      } yield "Ok"
    }
  } yield dl

  def spec =
    suite("DataLoaderSpec")(test("fail first") {
      for {
        dl <- failsFirstDL
        r1 <- dl.load(1).either
        r2 <- dl.load(1).either
      } yield assertTrue(r1 == Left("Failure") && r2 == Right("Ok"))
    }) @@ timeout(5 seconds)

}
