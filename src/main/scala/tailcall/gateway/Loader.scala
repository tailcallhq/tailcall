package tailcall.gateway

import tailcall.gateway.adt.Config
import zio.json._
import zio.json.yaml._
import zio.{Task, ZIO}

import scala.io.Source

object Loader {
  sealed trait Extension {
    self =>
    def decode[A](string: String): Task[Config] = ZIO
      .fromEither(
        self match {
          case Extension.JSON =>
            string.fromJson[Config]
          case Extension.YML  =>
            string.fromYaml[Config]
        },
      )
      .mapError(new RuntimeException(_))

    def encode(config: Config): Task[String] = ZIO
      .fromEither(
        self match {
          case Extension.JSON =>
            Right(config.toJsonPretty)
          case Extension.YML  =>
            config.toYaml(YamlOptions.default.copy(sequenceIndentation = 0))
        },
      )
      .mapError(new RuntimeException(_))
  }

  object Extension {
    case object JSON extends Extension
    case object YML  extends Extension
    def detect(name: String): Task[Extension] = {
      if (name.endsWith(".json"))
        ZIO.succeed(JSON)
      else if (name.endsWith(".yml"))
        ZIO.succeed(YML)
      else
        ZIO.fail(new RuntimeException(s"Unsupported file format: ${name}"))
    }
  }

  def readFile(name: String): Task[String] = ZIO
    .attemptBlocking(Source.fromResource(name).mkString(""))

  def load(name: String): Task[Config] = {
    for {
      ext      <- Extension.detect(name)
      input    <- readFile(name)
      endpoint <- ext.decode(input)
    } yield endpoint
  }
}
