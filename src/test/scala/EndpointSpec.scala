import com.tailcall.gateway.adt.Endpoint
import zio.json.{DecoderOps, EncoderOps}
import zio.test._
import zio.{Scope, Task, ZIO}

import scala.io.Source

object EndpointSpec extends ZIOSpecDefault {
  def read(name: String): Task[String] = {
    ZIO.attemptBlocking(Source.fromResource(name).mkString(""))
  }

  def parse(string: String): ZIO[Any, Throwable, Endpoint] = {
    ZIO.fromEither(string.fromJson[Endpoint]).mapError(new RuntimeException(_))
  }

  override def spec: Spec[TestEnvironment with Scope, Any] = suite("endpoint")(test("codec") {
    for {
      input     <- read("Endpoints.json")
      endpoint0 <- parse(input)
      endpoint1 <- parse(endpoint0.toJson)
    } yield assertTrue(endpoint0 == endpoint1)
  })
}
