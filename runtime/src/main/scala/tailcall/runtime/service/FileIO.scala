package tailcall.runtime.service

import zio.{Task, ZIO, ZLayer}

import java.io.File
import java.nio.file.Files

trait FileIO {
  def read(file: File): Task[String]
  def write(file: File, content: String): Task[Unit]
}

object FileIO {
  def read(file: File): ZIO[FileIO, Throwable, String]                 = ZIO.serviceWithZIO(_.read(file))
  def write(file: File, content: String): ZIO[FileIO, Throwable, Unit] = ZIO.serviceWithZIO(_.write(file, content))

  def live: ZLayer[Any, Nothing, FileIO] =
    ZLayer.succeed(new FileIO {
      override def read(file: File): Task[String]                 = ZIO.attemptBlocking(Files.readString(file.toPath))
      override def write(file: File, content: String): Task[Unit] =
        ZIO.attemptBlocking(Files.write(file.toPath, content.getBytes)).unit
    })
}
