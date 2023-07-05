package tailcall.runtime.service

import tailcall.runtime.model.Postman
import tailcall.runtime.model.Postman._
import zio.json.DecoderOps
import zio.{Task, ZIO, ZLayer}

import java.io.File
import java.net.URL

trait PostmanIO {
  def read(url: URL): Task[Postman]
  def read(file: File): Task[Postman] = read(file.toURI.toURL)
}

object PostmanIO {
  def default: ZLayer[Any, Nothing, PostmanIO] = FileIO.default >>> live

  def live: ZLayer[FileIO, Nothing, PostmanIO] = ZLayer.fromFunction(Live.apply _)

  def read(url: URL): ZIO[PostmanIO, Throwable, Postman] = ZIO.serviceWithZIO(_.read(url))

  final case class Live(fileIO: FileIO) extends PostmanIO {
    override def read(url: URL): Task[Postman] = {
      for {
        file    <- fileIO.read(new File(url.getFile))
        postman <- parse(file).mapError(new RuntimeException(_))
      } yield postman
    }

    private def parse(file: String): ZIO[Any, String, Postman] = {
      val asPostman    = ZIO.fromEither(file.fromJson[Postman])
      val asCollection = ZIO.fromEither(file.fromJson[Collection])

      asCollection.map(Postman(_)) <> asPostman
    }
  }
}
