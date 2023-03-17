package tailcall.runtime.service

import tailcall.runtime.dsl.json.Config
import zio.{Task, ZLayer}

import java.io.File

trait ConfigFileReader {
  def read(file: File): Task[Config]
}

object ConfigFileReader {
  final case class Live(fileIO: FileIO) extends ConfigFileReader {
    override def read(file: File): Task[Config] =
      for {
        ext    <- Extension.detect(file.getName)
        string <- fileIO.read(file)
        config <- ext.decode(string)
      } yield config
  }

  def live: ZLayer[FileIO, Nothing, ConfigFileReader] = ZLayer.fromFunction(Live.apply _)
}
