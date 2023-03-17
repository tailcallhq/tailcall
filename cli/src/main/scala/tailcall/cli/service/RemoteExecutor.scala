package tailcall.cli.service

import tailcall.runtime.ast.{Blueprint, Digest}
import zio.{Task, ZIO, ZLayer}

import java.net.InetAddress

/**
 * A service that can execute commands on a remote Tailcall
 * instance.
 */
trait RemoteExecutor {
  def deploy(address: InetAddress, blueprint: Blueprint): Task[Unit]
  def drop(address: InetAddress, digest: Digest): Task[Unit]
  def activate(address: InetAddress, digest: Digest): Task[Unit]
  def deactivate(address: InetAddress, digest: Digest): Task[Unit]
  def list(address: InetAddress): Task[List[Blueprint]]
  def load(address: InetAddress, digest: Digest): Task[Option[Blueprint]]
}

object RemoteExecutor {
  final case class Live() extends RemoteExecutor {
    override def deploy(address: InetAddress, blueprint: Blueprint): Task[Unit]      = ???
    override def drop(address: InetAddress, digest: Digest): Task[Unit]              = ???
    override def activate(address: InetAddress, digest: Digest): Task[Unit]          = ???
    override def deactivate(address: InetAddress, digest: Digest): Task[Unit]        = ???
    override def list(address: InetAddress): Task[List[Blueprint]]                   = ???
    override def load(address: InetAddress, digest: Digest): Task[Option[Blueprint]] = ???
  }

  def deploy(address: InetAddress, blueprint: Blueprint): ZIO[RemoteExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO(_.deploy(address, blueprint))

  def drop(address: InetAddress, digest: Digest): ZIO[RemoteExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO(_.drop(address, digest))

  def activate(address: InetAddress, digest: Digest): ZIO[RemoteExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO(_.activate(address, digest))

  def deactivate(address: InetAddress, digest: Digest): ZIO[RemoteExecutor, Throwable, Unit] =
    ZIO.serviceWithZIO(_.deactivate(address, digest))

  def list(address: InetAddress): ZIO[RemoteExecutor, Throwable, List[Blueprint]] = ZIO.serviceWithZIO(_.list(address))

  def load(address: InetAddress, digest: Digest): ZIO[RemoteExecutor, Throwable, Option[Blueprint]] =
    ZIO.serviceWithZIO(_.load(address, digest))

  def live: ZLayer[Any, Nothing, RemoteExecutor] = { ZLayer.succeed(Live()) }
}
