import caliban.parsing.Parser
import caliban.parsing.adt.Document
import com.tailcall.gateway.adt.Endpoint
import zio._

import scala.io.Source

object Example extends ZIOAppDefault {

  def readResource(name: String): Task[String] = ZIO
    .attemptBlocking(Source.fromResource(name).mkString(""))

  private def readSchema: Task[Document] = for {
    file   <- readResource("Schema.graphql")
    result <- Parser.parseQuery(file)
  } yield result

  import zio.json.yaml._

  private def readEndpoints: Task[Endpoint] = for {
    file   <- readResource("Endpoints.yml")
    result <- file.fromYaml[Endpoint] match {
      case Left(value)  => ZIO.fail(new RuntimeException(value))
      case Right(value) => ZIO.succeed(value)
    }
  } yield result

  val run = readEndpoints.tap(result => ZIO.succeed(pprint.pprintln(result)))
}
