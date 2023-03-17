package tailcall.runtime.service

import tailcall.runtime.dsl.json.Config
import zio.{Task, ZIO, ZLayer}

import java.io.File
import java.net.URL
trait ConfigFileReader {
  def read(file: File): Task[Config]
}

object ConfigFileReader {
  def readURL(url: URL): ZIO[ConfigFileReader, Throwable, Config] = readFile(new File(url.getPath))

  def readFile(file: File): ZIO[ConfigFileReader, Throwable, Config] = ZIO.serviceWithZIO(_.read(file))

  def live: ZLayer[FileIO, Nothing, ConfigFileReader] = ZLayer.fromFunction(Live.apply _)

  final case class Live(fileIO: FileIO) extends ConfigFileReader {
    override def read(file: File): Task[Config] =
      for {
        ext    <- Extension.detect(file.getName)
        string <- fileIO.read(file)
        config <- ext.decode(string)
      } yield config
  }
}
