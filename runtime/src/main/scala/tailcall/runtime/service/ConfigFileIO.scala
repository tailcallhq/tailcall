package tailcall.runtime.service

import tailcall.runtime.dsl.json.Config
import zio.{Task, ZIO, ZLayer}

import java.io.File
import java.net.URL
trait ConfigFileIO {
  self =>
  def read(file: File): Task[Config]
  def readAll(files: List[File]): Task[Config] =
    ZIO.foreach(files)(file => self.read(file)).map(_.reduce(_ mergeRight _))
  def write(file: File, config: Config): Task[Unit]
}

object ConfigFileIO {
  def readURL(url: URL): ZIO[ConfigFileIO, Throwable, Config]           = readFile(new File(url.getPath))
  def readFile(file: File): ZIO[ConfigFileIO, Throwable, Config]        = ZIO.serviceWithZIO(_.read(file))
  def live: ZLayer[FileIO with GraphQLGenerator, Nothing, ConfigFileIO] = ZLayer.fromFunction(Live.apply _)
  def default: ZLayer[Any, Nothing, ConfigFileIO] = (FileIO.default ++ GraphQLGenerator.default) >>> live

  final case class Live(fileIO: FileIO, graphQLGenerator: GraphQLGenerator) extends ConfigFileIO {
    override def read(file: File): Task[Config] =
      for {
        ext    <- DSLFormat.detect(file.getName).mapError(new RuntimeException(_))
        string <- fileIO.read(file)
        config <- ext.decode(string).mapError(new RuntimeException(_))
      } yield config

    override def write(file: File, config: Config): Task[Unit] =
      for {
        ext    <- DSLFormat.detect(file.getName).mapError(new RuntimeException(_))
        string <- ext.encode(config).mapError(new RuntimeException(_))
        _      <- fileIO.write(file, string)
      } yield ()
  }
}
