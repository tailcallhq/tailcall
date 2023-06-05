package tailcall.test

import zio.test.{TestAspect, TestAspectAtLeastR, TestEnvironment, ZIOSpecDefault}
import zio.{Chunk, durationInt}

trait TailcallSpec extends ZIOSpecDefault {
  self =>
  override def aspects: Chunk[TestAspectAtLeastR[TestEnvironment]] =
    super.aspects :+ TestAspect.timed :+ TestAspect.timeout(10 seconds)
}
