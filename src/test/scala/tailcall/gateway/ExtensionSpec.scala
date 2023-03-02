package tailcall.gateway

import tailcall.gateway.internal.FileExtension
import zio.test.TestAspect.failing
import zio.test._
import zio.{Task, ZIO}

import scala.io.Source

object ExtensionSpec extends ZIOSpecDefault:

  def read(file: String): Task[String] = ZIO.attemptBlocking(Source.fromResource(file).mkString(""))

  // TODO: fix failing tests
  def spec =
    suite("ExtensionSpec")(test("json codec") {
      val gen = Gen.fromIterable(Seq(FileExtension.YML, FileExtension.JSON))
      checkAll(gen) { ext =>
        for {
          str     <- read(s"Config.${ext.name}")
          config  <- ext.decode(str)
          str0    <- ext.encode(config)
          config0 <- ext.decode(str0)
        } yield assertTrue(config0 == config)
      }
    }) @@ failing
