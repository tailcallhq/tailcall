package tailcall.test

import zio.test.{TestAspect, TestAspectAtLeastR, TestEnvironment, ZIOSpecDefault, testEnvironment}
import zio.{Chunk, LogLevel, ZLayer, ZLogger, durationInt}

trait TailcallSpec extends ZIOSpecDefault {
  self =>
  override val bootstrap: ZLayer[Any, Any, TestEnvironment] = testEnvironment ++ zio.Runtime
    .removeDefaultLoggers ++ ZLayer.succeed(ZLogger.default.filterLogLevel(_ >= LogLevel.Warning))

  override def aspects: Chunk[TestAspectAtLeastR[TestEnvironment]] =
    super.aspects :+ TestAspect.timed :+ TestAspect.timeout(10 seconds) :+ TestAspect.parallel :+ TestAspect.silent
}
