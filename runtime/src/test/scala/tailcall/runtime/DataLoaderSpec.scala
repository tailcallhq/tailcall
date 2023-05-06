package tailcall.runtime

import tailcall.runtime.service.DataLoader
import zio._
import zio.test.TestAspect.{nonFlaky, timeout}
import zio.test._

object DataLoaderSpec extends ZIOSpecDefault {
  def spec =
    suite("DataLoaderSpec")(
      test("fail first") {
        for {
          dl <- for {
            ref <- Ref.make(true)
            dl  <- DataLoader.one[Int] { _ =>
              for {
                fail <- ref.get
                _    <- ref.set(false)
                _    <- ZIO.fail("Failure").when(fail)
              } yield "Ok"
            }
          } yield dl
          f1 <- dl.collect(1)
          _  <- dl.dispatch
          r1 <- f1.either
          f2 <- dl.collect(1)
          _  <- dl.dispatch
          r2 <- f2.either
        } yield assertTrue(r1 == Left("Failure") && r2 == Right("Ok"))
      } @@ nonFlaky,
      test("should cache") {
        for {
          dl  <- DataLoader.one[Int](value => zio.Console.print("Load") *> ZIO.succeed(value + 1))
          f1  <- dl.collect(1)
          _   <- dl.dispatch
          f2  <- dl.collect(1)
          _   <- dl.dispatch
          r1  <- f1
          r2  <- f2
          out <- TestConsole.output
        } yield assertTrue(r1 == 2 && r2 == 2, out == Vector("Load"))
      },
      test("concurrent load") {
        for {
          dl  <- DataLoader.one[Int](_ => zio.Console.print("Load"))
          _   <- ZIO.foreachParDiscard(0 to 100)(_ => dl.load(1))
          out <- TestConsole.output
        } yield assertTrue(out == Vector("Load"))
      } @@ nonFlaky,
      test("multi request") {
        for {
          dl  <- DataLoader.one[Int](_ => zio.Console.print("Load").delay(5 second))
          _   <- dl.load(1).fork
          _   <- TestClock.adjust(5 second)
          f1  <- dl.load(2).fork
          _   <- TestClock.adjust(1 second)
          f2  <- dl.load(2).fork
          _   <- TestClock.adjust(5 second)
          _   <- f1.join.zip(f2.join)
          out <- TestConsole.output
        } yield assertTrue(out == Vector("Load", "Load"))
      },
      test("multi request concurrent") {
        for {
          dl  <- DataLoader.one[Int](_ => zio.Console.print("Load").delay(5 second))
          f1  <- dl.load(1).fork
          f2  <- dl.load(2).fork
          _   <- TestClock.adjust(5 second)
          _   <- f1.join.zip(f2.join)
          out <- TestConsole.output
        } yield assertTrue(out == Vector("Load", "Load"))
      },
      test("batch") {
        val value: UIO[DataLoader[Any, Nothing, Int, Int]] = DataLoader
          .many[Int](chunk => ZIO.succeed(chunk.map(_ + 1)))

        for {
          dl <- value
          f  <- dl.collect(1, 2, 3, 4)
          _  <- dl.dispatch
          r  <- ZIO.foreach(f)(identity)
        } yield assertTrue(r == Chunk(2, 3, 4, 5))
      },
    ) @@ timeout(5 seconds)

}
